//! Body adapters for the shared spell resolver.
use super::{
    BodySpellResult, DirectSpell, FieldUsage, RandomState, RestrictionMemoryView, RuleError,
    SupportSpellResult, resolve_support_inner,
};
use centurions_content::{Warrior, combat::MonsterRecord};

/// A spell cast on the warriors and monsters its mask picks.
pub fn resolve_bodies(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_bodies_inner(warriors, enemies, mask, context, None, None, random)
}

/// A spell cast on the picked bodies, with the item definition last loaded.
pub fn resolve_bodies_with_item_cache(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    item_cache: &[u8],
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_bodies_inner(
        warriors,
        enemies,
        mask,
        context,
        None,
        Some(RestrictionMemoryView::item_cache_only(item_cache)),
        random,
    )
}

/// A spell cast on the picked bodies, with the party's restriction memory.
pub fn resolve_bodies_with_memory(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    memory: RestrictionMemoryView<'_>,
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_bodies_inner(warriors, enemies, mask, context, None, Some(memory), random)
}

/// A spell field strikes the bodies its mask picks and is marked used.
pub fn resolve_collision(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    usage: &mut FieldUsage,
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_collision_inner(
        warriors,
        enemies,
        mask,
        context,
        usage,
        ((None, None), true),
        random,
    )
    .map(|result| result.body)
}

/// A spell field strikes the picked bodies, with the item definition last loaded.
pub fn resolve_collision_with_item_cache(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    usage: &mut FieldUsage,
    item_cache: &[u8],
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_collision_inner(
        warriors,
        enemies,
        mask,
        context,
        usage,
        (
            (
                None,
                Some(RestrictionMemoryView::item_cache_only(item_cache)),
            ),
            true,
        ),
        random,
    )
    .map(|result| result.body)
}

/// A spell field strikes the picked bodies, with the current body and the party's restriction
/// memory. Also gives what the field recovered or healed.
pub(crate) fn resolve_collision_with_memory(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    usage: &mut FieldUsage,
    caller: (Option<i16>, RestrictionMemoryView<'_>),
    random: &mut RandomState,
) -> Result<CollisionResult, RuleError> {
    resolve_collision_inner(
        warriors,
        enemies,
        mask,
        context,
        usage,
        ((caller.0, Some(caller.1)), false),
        random,
    )
}

/// What a field's strike did: the body result, and what it recovered or healed.
pub(crate) struct CollisionResult {
    pub body: BodySpellResult,
    pub recovered: u8,
    pub healed: bool,
}

/// A spell field strikes bodies. A psi-drain field (effect 60 or 61, defense category 5) drains psi
/// with its usage byte on behalf of the current body; any other field is resolved as a support
/// spell. The usage marks are kept only if the strike succeeds.
pub(super) fn resolve_collision_inner(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    usage: &mut FieldUsage,
    (caller, record_only): ((Option<i16>, Option<RestrictionMemoryView<'_>>), bool),
    random: &mut RandomState,
) -> Result<CollisionResult, RuleError> {
    if usage.slot >= 60 || usage.used.len() != 128 {
        return Err(RuleError::Invalid("spell field collision usage".into()));
    }
    let mut next_usage = FieldUsage {
        slot: usage.slot,
        used: usage.used.clone(),
    };
    let attack = centurions_content::records::SpellInfoRecord(context.packet);
    if matches!(attack.effect()?, 60 | 61) && attack.defense_category() == 5 {
        // A field collision drains Psi with its usage byte. Every game collision
        // passes the party memory and the current body; the record-only public
        // resolvers, which no command reaches, pass neither.
        let Some(memory) = caller.1.filter(|_| !record_only) else {
            return Err("category-5 drain in a record-only collision resolver".into());
        };
        // The caster is the battle order's current body; a negative row reads
        // before the warrior table (docs/memory-alias-audit.md).
        let Some(caster) = caller.0 else {
            return Err(RuleError::Unsupported {
                action: "combat.spell.collision",
                dependency: "category-5 drain collision without a current initiative body",
            });
        };
        let body = super::resolve_psi_drain(
            warriors,
            enemies,
            mask,
            context,
            caster,
            (memory, Some(&mut next_usage)),
            random,
        )?;
        *usage = next_usage;
        return Ok(CollisionResult {
            body,
            recovered: 0,
            healed: false,
        });
    }
    let result = resolve_support_inner(
        warriors,
        enemies,
        mask,
        context,
        caller,
        Some(&mut next_usage),
        random,
    )?;
    no_alias_writes(&result)?;
    let (recovered, healed) = (result.recovered, result.healed);
    *usage = next_usage;
    Ok(CollisionResult {
        body: body_result(result),
        recovered,
        healed,
    })
}

/// Resolves a spell on bodies as a support spell and keeps the body results.
pub(super) fn resolve_bodies_inner(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    usage: Option<&mut FieldUsage>,
    memory: Option<RestrictionMemoryView<'_>>,
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    resolve_support_inner(
        warriors,
        enemies,
        mask,
        context,
        (None, memory),
        usage,
        random,
    )
    .and_then(|result| {
        no_alias_writes(&result)?;
        Ok(body_result(result))
    })
}

/// Support packets with effects 26 and 64 reach these callers without the item,
/// shop or map owners, so they record no warrior-indexed write.
fn no_alias_writes(result: &SupportSpellResult) -> Result<(), RuleError> {
    if result.alias_writes.is_empty() {
        Ok(())
    } else {
        Err("warrior-indexed write without retained owners".into())
    }
}

/// Keeps the body part of a support result.
fn body_result(result: SupportSpellResult) -> BodySpellResult {
    BodySpellResult {
        total_delta: result.total_delta,
        warriors: result.warriors,
        enemies: result.enemies,
        sounds: result.sounds,
        targets: result.targets,
        race_excluded: result.race_excluded,
    }
}
