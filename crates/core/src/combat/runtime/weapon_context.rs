//! The weapon a body has ready.
use crate::RuleError;
use crate::combat::Combat;
use centurions_content::Warrior;

/// Whether the item is of the melee weapon class (item class 2); item 0 and ids outside the item
/// table are not.
pub(super) fn is_melee_weapon(items: Option<&[Vec<u8>]>, id: i16) -> Result<bool, RuleError> {
    let Some(definition) = usize::try_from(id)
        .ok()
        .filter(|id| *id != 0)
        .and_then(|id| items.and_then(|items| items.get(id)))
    else {
        return Ok(false);
    };
    Ok(centurions_content::records::ItemDefinition(definition).item_type()? == 2)
}

/// Loads the natural attack as the weapon in use.
pub(super) fn load_natural_attack(
    combat: &mut Combat,
    items: Option<&[Vec<u8>]>,
    buffers: &mut crate::ammunition::LoadedItems,
) -> Result<(), RuleError> {
    if let Some(items) = items {
        let definition = items
            .first()
            .ok_or_else(|| RuleError::Invalid("natural weapon definition missing".into()))?;
        combat.weapon_traits.load_pair(definition, definition)?;
        buffers.load_natural(items)?;
    } else {
        *buffers = Default::default();
    }
    Ok(())
}

/// Loads a monster's melee weapon as the weapon in use when it holds one.
pub(super) fn load_melee_weapon(
    combat: &mut Combat,
    attacker: i16,
    weapon: &[u8],
    items: Option<&[Vec<u8>]>,
    buffers: &mut crate::ammunition::LoadedItems,
) -> Result<(), RuleError> {
    if combat.roster.records[(attacker - 10) as usize]
        .record()
        .weapon()?
        != 0
    {
        combat.weapon_traits.load_weapon(weapon)?;
        if let Some(items) = items {
            buffers.load_weapon(
                items,
                combat.roster.records[(attacker - 10) as usize]
                    .record()
                    .melee_weapon()?,
            )?;
        } else {
            buffers.weapon = None;
        }
        Ok(())
    } else {
        load_natural_attack(combat, items, buffers)
    }
}

/// Reloads the weapon a body has ready: a warrior's second weapon in firing mode, otherwise its
/// melee weapon; a monster's selected weapon.
pub(crate) fn refresh_body_item(
    combat: &Combat,
    warriors: &mut [Warrior],
    items: &[Vec<u8>],
    body: i16,
    buffers: &mut crate::ammunition::LoadedItems,
) -> Result<(), RuleError> {
    let id = if (0..6).contains(&body) {
        let warrior = &mut warriors[body as usize];
        let secondary = warrior.record().toggle();
        let id = if secondary {
            warrior.record().equipped_weapon()?
        } else {
            warrior.record().melee_weapon()?
        };
        if id == 0 {
            // Only the active body loads item 0; a missing second weapon then clears
            // firing mode.
            if combat.active_body() == Some(body) {
                buffers.load_item_from(items, id)?;
            }
            if secondary {
                warrior.record_mut().set_toggle(false);
            }
            return Ok(());
        }
        id
    } else if body >= 10 {
        let id = combat.roster.records[(body - 10) as usize]
            .record()
            .weapon()?;
        if id == 0 {
            return Ok(());
        }
        id
    } else {
        return Err(RuleError::Invalid(
            "item refresh body outside represented actors".into(),
        ));
    };
    buffers.load_item_from(items, id)?;
    Ok(())
}
