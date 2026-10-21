/// Condition slots are shared by warrior WORDs and enemy signed BYTEs. Names
/// follow Cargo Pod STR#133; blank original labels use their observed rule role.
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
    UnlabelledDefensePenalty = 7,
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
