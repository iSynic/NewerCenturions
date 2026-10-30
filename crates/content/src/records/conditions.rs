/// The forty condition slots, held as words by warriors and signed bytes by
/// monsters. Names follow Cargo Pod STR#133; a slot with a blank label is named
/// for what the rules do with it.
#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Condition {
    Retreating = 0,
    Helpless = 1,
    Entangled = 2,
    Poisoned = 3,
    Regenerating = 4,
    ScanBlocked = 5,
    MovingSlowly = 6,
    ShieldedFromHits = 7,
    SeventhLevelProtection = 22,
    Speedy = 23,
    ComputerControl = 25,
    Blind = 27,
    Diseased = 28,
    Confused = 29,
    ReflectingScans = 30,
    PhysicalReflection = 31,
    BonusDamage = 32,
    RechargePsi = 33,
    DrainPsi = 34,
    HinderedAttacks = 36,
    HinderedDefense = 37,
    IncreasedDefense = 38,
    UnlabelledScanBlock = 39,
}
