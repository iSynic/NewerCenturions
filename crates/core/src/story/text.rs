//! Names in authored story text.
use crate::RuleError;
use centurions_content::Warrior;

/// Fills a story text's name tokens. Each `^` and the character after it name a squad member
/// (`^1` the first, `^2` the second and so on). A member who has fallen, or who is not in the squad,
/// gives way to the last member still standing. A token that still names no member in the squad
/// (`^7`, `^8` or any other character) gives the first member's name. Both characters of a token
/// are replaced at once, and the name put in is never read as a token itself.
///
/// Original: textbox
pub fn substitute_names(text: &str, warriors: &[Warrior]) -> Result<String, RuleError> {
    /// Whether the squad member in that place is still standing.
    fn standing(warriors: &[Warrior], member: i8) -> Result<bool, RuleError> {
        match usize::try_from(member).ok().and_then(|i| warriors.get(i)) {
            Some(warrior) => Ok(warrior.stamina()? > 0),
            None => Ok(false),
        }
    }
    let mut filled = String::with_capacity(text.len());
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character != '^' {
            filled.push(character);
            continue;
        }
        let Some(token) = characters.next() else {
            filled.push(character);
            break;
        };
        let code = u8::try_from(u32::from(token)).unwrap_or(0xff);
        let mut member = code.wrapping_sub(b'1') as i8;
        if !standing(warriors, member)? {
            for candidate in (0..warriors.len()).rev() {
                if standing(warriors, candidate as i8)? {
                    member = candidate as i8;
                    break;
                }
            }
        }
        let place = usize::try_from(member)
            .ok()
            .filter(|place| *place < warriors.len().min(6))
            .unwrap_or(0);
        if let Some(warrior) = warriors.get(place) {
            filled.push_str(&warrior.name);
        }
    }
    Ok(filled)
}
