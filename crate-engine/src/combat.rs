//! Damage/kill resolution — Path A of the wasm-ecs migration (see
//! `.claude/plans/agile-shimmying-metcalfe.md` at the time this was written,
//! and `.tools/docs/engineering/WASM_ECS_MIGRATION_PLAN.md`'s Phase 3
//! step 5). Mirrors `index.html`'s `dealDamage()`/`killEnemy()` — pure
//! functions only, no ECS/component state, no side effects (SFX/particles/
//! logEvent/achievements stay JS-only, driven by what these functions say
//! happened, not duplicated here — the fidelity scope explicitly excludes
//! presentation-only branches).
//!
//! Milestone 1 (this module's first cut): the pre-crit multiplier chain —
//! elite/boss, slowed-target, execute, berserk, momentum cap, turret mode.
//! No RNG yet (crit roll + on-hit procs are Milestone 2, once this is
//! proven).

/// Mirrors `dealDamage()`'s multiplier chain exactly, up to (not including)
/// the crit roll — `index.html`'s:
/// ```js
/// if(isEliteOrBoss) dmg *= (player.stats.eliteDmgMult||1);
/// if(enemy.slowed) dmg *= (player.stats.dmgToSlowedMult||1);
/// if(enemy.hp/enemy.maxHp < 0.2) dmg *= (1+(player.stats.executeBonus||0));
/// if(player.stats.berserkScaling) dmg *= (1 + player.stats.berserkScaling*(100*(1-player.hp/player.maxHp)));
/// if(player.momentumStacks) dmg *= (1+(player.stats.momentumPerKill||0)*Math.min(player.momentumStacks,400));
/// if(player.stats.turretMode && /* stationary this frame */) dmg *= 1.2;
/// ```
///
/// Callers pass already-defaulted stat values (JS's `||1`/`||0` fallbacks
/// happen at the call site, same convention as every other bridge
/// function this session — e.g. `chaseSeekVelocity`'s `e.chaseTime||0`).
/// The berserk/momentum terms are applied unconditionally rather than
/// gated on `if(stat)` like the JS — mathematically identical, since a
/// zero stat multiplies by exactly 1 either way, and it keeps this
/// function branch-free where JS's guard was only ever a micro-optimization,
/// not a behavioral difference.
pub fn damage_multiplier_chain(
    base_dmg: f32,
    is_elite_or_boss: bool,
    elite_dmg_mult: f32,
    enemy_slowed: bool,
    dmg_to_slowed_mult: f32,
    enemy_hp_fraction: f32,
    execute_bonus: f32,
    berserk_scaling: f32,
    player_hp_fraction: f32,
    momentum_per_kill: f32,
    momentum_stacks: f32,
    turret_mode_active: bool,
) -> f32 {
    const EXECUTE_HP_THRESHOLD: f32 = 0.2;
    const MOMENTUM_STACK_CAP: f32 = 400.0;
    const TURRET_MODE_BONUS: f32 = 1.2;

    let mut dmg = base_dmg;
    if is_elite_or_boss {
        dmg *= elite_dmg_mult;
    }
    if enemy_slowed {
        dmg *= dmg_to_slowed_mult;
    }
    if enemy_hp_fraction < EXECUTE_HP_THRESHOLD {
        dmg *= 1.0 + execute_bonus;
    }
    dmg *= 1.0 + berserk_scaling * (100.0 * (1.0 - player_hp_fraction));
    dmg *= 1.0 + momentum_per_kill * momentum_stacks.min(MOMENTUM_STACK_CAP);
    if turret_mode_active {
        dmg *= TURRET_MODE_BONUS;
    }
    dmg
}

#[cfg(test)]
mod tests {
    use super::*;

    // Independent JS-equivalent recomputation, kept deliberately separate
    // from the function under test (not just calling it back) — the same
    // discipline used for every diagnostic ground-truth check this session.
    fn js_multiplier_chain(
        base_dmg: f32,
        is_elite_or_boss: bool,
        elite_dmg_mult: f32,
        enemy_slowed: bool,
        dmg_to_slowed_mult: f32,
        enemy_hp_fraction: f32,
        execute_bonus: f32,
        berserk_scaling: f32,
        player_hp_fraction: f32,
        momentum_per_kill: f32,
        momentum_stacks: f32,
        turret_mode_active: bool,
    ) -> f32 {
        let mut dmg = base_dmg;
        if is_elite_or_boss {
            dmg *= elite_dmg_mult;
        }
        if enemy_slowed {
            dmg *= dmg_to_slowed_mult;
        }
        if enemy_hp_fraction < 0.2 {
            dmg *= 1.0 + execute_bonus;
        }
        if berserk_scaling != 0.0 {
            dmg *= 1.0 + berserk_scaling * (100.0 * (1.0 - player_hp_fraction));
        }
        if momentum_stacks != 0.0 {
            dmg *= 1.0 + momentum_per_kill * momentum_stacks.min(400.0);
        }
        if turret_mode_active {
            dmg *= 1.2;
        }
        dmg
    }

    fn assert_close(a: f32, b: f32, label: &str) {
        assert!((a - b).abs() < 0.001, "{label}: {a} != {b}");
    }

    #[test]
    fn baseline_no_modifiers() {
        let got = damage_multiplier_chain(100.0, false, 1.0, false, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, false);
        assert_close(got, 100.0, "baseline");
    }

    #[test]
    fn elite_boss_multiplier() {
        let got = damage_multiplier_chain(100.0, true, 1.5, false, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, false);
        let want = js_multiplier_chain(100.0, true, 1.5, false, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, false);
        assert_close(got, want, "elite/boss");
        assert_close(got, 150.0, "elite/boss exact");
    }

    #[test]
    fn slowed_target_bonus() {
        let got = damage_multiplier_chain(100.0, false, 1.0, true, 1.3, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, false);
        assert_close(got, 130.0, "slowed");
    }

    #[test]
    fn execute_bonus_below_threshold() {
        let got = damage_multiplier_chain(100.0, false, 1.0, false, 1.0, 0.15, 0.5, 0.0, 1.0, 0.0, 0.0, false);
        assert_close(got, 150.0, "execute triggers under 20% hp");
    }

    #[test]
    fn execute_bonus_at_and_above_threshold_does_not_trigger() {
        let at = damage_multiplier_chain(100.0, false, 1.0, false, 1.0, 0.2, 0.5, 0.0, 1.0, 0.0, 0.0, false);
        assert_close(at, 100.0, "execute does not trigger AT exactly 20% (strict <)");
        let above = damage_multiplier_chain(100.0, false, 1.0, false, 1.0, 0.5, 0.5, 0.0, 1.0, 0.0, 0.0, false);
        assert_close(above, 100.0, "execute does not trigger above 20%");
    }

    #[test]
    fn berserk_scaling_at_low_player_hp() {
        // player at 10% hp, berserkScaling=0.01 -> 1 + 0.01*(100*0.9) = 1.9x
        let got = damage_multiplier_chain(100.0, false, 1.0, false, 1.0, 1.0, 0.0, 0.01, 0.1, 0.0, 0.0, false);
        assert_close(got, 190.0, "berserk scaling");
    }

    #[test]
    fn momentum_stack_cap_applies() {
        // 1000 stacks should behave identically to the 400 cap
        let uncapped_input = damage_multiplier_chain(100.0, false, 1.0, false, 1.0, 1.0, 0.0, 0.0, 1.0, 0.01, 1000.0, false);
        let at_cap = damage_multiplier_chain(100.0, false, 1.0, false, 1.0, 1.0, 0.0, 0.0, 1.0, 0.01, 400.0, false);
        assert_close(uncapped_input, at_cap, "momentum cap");
        assert_close(at_cap, 100.0 * (1.0 + 0.01 * 400.0), "momentum cap exact value");
    }

    #[test]
    fn turret_mode_bonus() {
        let got = damage_multiplier_chain(100.0, false, 1.0, false, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, true);
        assert_close(got, 120.0, "turret mode");
    }

    #[test]
    fn all_modifiers_stack_multiplicatively_matches_js_ground_truth() {
        let args = (250.0_f32, true, 1.3_f32, true, 1.2_f32, 0.15_f32, 0.4_f32, 0.02_f32, 0.3_f32, 0.005_f32, 250.0_f32, true);
        let got = damage_multiplier_chain(args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9, args.10, args.11);
        let want = js_multiplier_chain(args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9, args.10, args.11);
        assert_close(got, want, "combined modifiers vs independent JS ground truth");
    }
}
