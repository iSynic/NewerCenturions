//! Weapon context in the original combat execution order.
use crate::RuleError;
use crate::combat::Combat;
use centurions_content::Warrior;

pub(super) fn is_melee_weapon(items: Option<&[Vec<u8>]>, id: i16) -> Result<bool, RuleError> {
    let Some(definition) = usize::try_from(id)
        .ok()
        .filter(|id| *id != 0)
        .and_then(|id| items.and_then(|items| items.get(id)))
    else {
        return Ok(false);
    };
    Ok(centurions_content::records::ItemDefinition(definition).category()? == 2)
}

pub(super) fn load_natural_work(
    c: &mut Combat,
    items: Option<&[Vec<u8>]>,
    buffers: &mut crate::ammunition::WorkBuffers,
) -> Result<(), RuleError> {
    if let Some(items) = items {
        let definition = items
            .first()
            .ok_or_else(|| RuleError::Invalid("natural weapon definition missing".into()))?;
        c.weapon_work.load_pair(definition, definition)?;
        buffers.load_natural(items)?;
    } else {
        *buffers = Default::default();
    }
    Ok(())
}

pub(super) fn load_melee_work(
    c: &mut Combat,
    attacker: i16,
    weapon: &[u8],
    items: Option<&[Vec<u8>]>,
    buffers: &mut crate::ammunition::WorkBuffers,
) -> Result<(), RuleError> {
    if c.enemies.records[(attacker - 10) as usize]
        .record()
        .selected_item()?
        != 0
    {
        c.weapon_work.load_weapon(weapon)?;
        if let Some(items) = items {
            buffers.load_weapon(
                items,
                c.enemies.records[(attacker - 10) as usize]
                    .record()
                    .melee_weapon()?,
            )?;
        } else {
            buffers.weapon = None;
        }
        Ok(())
    } else {
        load_natural_work(c, items, buffers)
    }
}

pub(crate) fn refresh_body_item(
    combat: &Combat,
    warriors: &mut [Warrior],
    items: &[Vec<u8>],
    body: i16,
    buffers: &mut crate::ammunition::WorkBuffers,
) -> Result<(), RuleError> {
    let id = if (0..6).contains(&body) {
        let warrior = &mut warriors[body as usize];
        let secondary = warrior.record().firing_mode();
        let id = if secondary {
            warrior.record().equipped_weapon()?
        } else {
            warrior.record().melee_weapon()?
        };
        if id == 0 {
            // combatupdate2 loads item 0 only for the active actor (11f646/11f662);
            // its missing-secondary branch 11fb7a then clears the mode byte.
            if combat.active_body() == Some(body) {
                buffers.load_item_from(items, id)?;
            }
            if secondary {
                warrior.record_mut().set_firing_mode(false);
            }
            return Ok(());
        }
        id
    } else if body >= 10 {
        let id = combat.enemies.records[(body - 10) as usize]
            .record()
            .selected_item()?;
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
