//! Native interface control bodies; Toolbox observations and rendering stay explicit.

use crate::RuleError;
use crate::recovered_geometry::BodyRect;
use crate::recovered_records::RecordWrite;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub vertical: i16,
    pub horizontal: i16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InterfaceEffect {
    Write(RecordWrite),
    GetPort,
    GetPenState,
    SetPort(u32),
    PenMode(i16),
    PenSize {
        horizontal: i16,
        vertical: i16,
    },
    PenPixelPattern(u32),
    MoveTo(Point),
    LineTo(Point),
    SetPenState([u8; 18]),
    LocalToGlobal(Point),
    PointInRect {
        point: Point,
        rect: BodyRect,
    },
    NewRegion,
    CopyRegion {
        source: u32,
        destination: u32,
    },
    GetMainDevice,
    RectRegion {
        region: u32,
        rect: BodyRect,
    },
    SectRegion {
        source_a: u32,
        source_b: u32,
        destination: u32,
    },
    UnionRegion {
        source_a: u32,
        source_b: u32,
        destination: u32,
    },
    PaintOne {
        window: u32,
        region: u32,
    },
    DrawMenuBar,
    DrawPicture {
        handle: u32,
        rect: BodyRect,
    },
}

/// Pure observations of native memory and external Toolbox replies.
pub trait InterfaceServices {
    fn word(&mut self, address: u32) -> Result<i16, RuleError>;
    fn current_port(&mut self) -> Result<u32, RuleError>;
    fn pen_state(&mut self) -> Result<[u8; 18], RuleError>;
    fn local_to_global(&mut self, point: Point) -> Result<Point, RuleError>;
    fn point_in_rect(&mut self, point: Point, rect: BodyRect) -> Result<u8, RuleError>;
    fn new_region(&mut self) -> Result<u32, RuleError>;
    /// The Rect dereferenced at main-device handle + 0x22, if available.
    fn main_device_rect(&mut self) -> Result<BodyRect, RuleError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfacePlan {
    pub native_return: i32,
    pub effects: Vec<InterfaceEffect>,
}

fn write(effects: &mut Vec<InterfaceEffect>, address: u32, bytes: &[u8]) {
    effects.push(InterfaceEffect::Write(RecordWrite {
        address,
        bytes: bytes.to_vec(),
    }));
}

/// inwindow, 0x12946c. Native body BYTE nine selects enemy index minus one.
pub fn inwindow(
    body: i8,
    height: i16,
    services: &mut impl InterfaceServices,
) -> Result<InterfacePlan, RuleError> {
    let rect = BodyRect {
        top: 1,
        left: 1,
        bottom: height.wrapping_shl(2).wrapping_add(8),
        right: height.wrapping_mul(5).wrapping_add(8),
    };
    let address = if body < 9 {
        0x44709au32.wrapping_add((i32::from(body) * 4) as u32)
    } else {
        0x446f0au32.wrapping_add((i32::from(i16::from(body).wrapping_sub(10)) * 4) as u32)
    };
    // The first position WORD is stored at local -10, the second at local -12.
    let horizontal = services.word(address)?;
    let vertical = services.word(address.wrapping_add(2))?;
    let point = Point {
        vertical,
        horizontal,
    };
    let result = services.point_in_rect(point, rect)?;
    Ok(InterfacePlan {
        native_return: i32::from(result != 0),
        effects: vec![InterfaceEffect::PointInRect { point, rect }],
    })
}

/// centercursor, 0x131564: LocalToGlobal changes the point before low-memory writes.
pub fn centercursor(services: &mut impl InterfaceServices) -> Result<InterfacePlan, RuleError> {
    let local = Point {
        vertical: 176,
        horizontal: 176,
    };
    let global = services.local_to_global(local)?;
    let mut effects = vec![InterfaceEffect::LocalToGlobal(local)];
    let mut bytes = [0; 4];
    bytes[..2].copy_from_slice(&global.vertical.to_be_bytes());
    bytes[2..].copy_from_slice(&global.horizontal.to_be_bytes());
    write(&mut effects, 0x82c, &bytes);
    write(&mut effects, 0x828, &bytes);
    write(&mut effects, 0x8ce, &(-1i16).to_be_bytes());
    Ok(InterfacePlan {
        native_return: 0,
        effects,
    })
}

/// locationlog, 0x13154c: the complete native four-instruction body returns zero.
pub fn locationlog() -> i32 {
    0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonInput {
    pub rect: BodyRect,
    pub target_port: u32,
    pub dark_pattern: u32,
    pub light_pattern: u32,
}

fn button(
    input: ButtonInput,
    switch_port: i8,
    pressed: bool,
    services: &mut impl InterfaceServices,
) -> Result<InterfacePlan, RuleError> {
    let port = services.current_port()?;
    let pen = services.pen_state()?;
    let mut effects = vec![InterfaceEffect::GetPort, InterfaceEffect::GetPenState];
    if switch_port != 0 {
        effects.push(InterfaceEffect::SetPort(input.target_port));
    }
    effects.push(InterfaceEffect::PenMode(0));
    effects.push(InterfaceEffect::PenSize {
        horizontal: 1,
        vertical: 1,
    });
    effects.push(InterfaceEffect::PenPixelPattern(if pressed {
        input.dark_pattern
    } else {
        input.light_pattern
    }));
    let r = input.rect;
    effects.push(InterfaceEffect::MoveTo(Point {
        horizontal: r.left,
        vertical: r.bottom.wrapping_sub(1),
    }));
    effects.push(InterfaceEffect::LineTo(Point {
        horizontal: r.left,
        vertical: r.top,
    }));
    effects.push(InterfaceEffect::LineTo(Point {
        horizontal: r.right.wrapping_sub(1),
        vertical: r.top,
    }));
    effects.push(InterfaceEffect::PenPixelPattern(if pressed {
        input.light_pattern
    } else {
        input.dark_pattern
    }));
    effects.push(InterfaceEffect::MoveTo(Point {
        horizontal: r.right.wrapping_sub(1),
        vertical: r.top.wrapping_add(1),
    }));
    effects.push(InterfaceEffect::LineTo(Point {
        horizontal: r.right.wrapping_sub(1),
        vertical: r.bottom.wrapping_sub(1),
    }));
    effects.push(InterfaceEffect::LineTo(Point {
        horizontal: r.left.wrapping_add(1),
        vertical: r.bottom.wrapping_sub(1),
    }));
    effects.push(InterfaceEffect::SetPort(port));
    effects.push(InterfaceEffect::SetPenState(pen));
    Ok(InterfacePlan {
        native_return: 0,
        effects,
    })
}

/// downbutton, 0x153720: dark top/left, then light bottom/right.
pub fn downbutton(
    input: ButtonInput,
    switch_port: i8,
    services: &mut impl InterfaceServices,
) -> Result<InterfacePlan, RuleError> {
    button(input, switch_port, true, services)
}

/// upbutton, 0x1537ee: light top/left, then dark bottom/right.
pub fn upbutton(
    input: ButtonInput,
    switch_port: i8,
    services: &mut impl InterfaceServices,
) -> Result<InterfacePlan, RuleError> {
    button(input, switch_port, false, services)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuBarContext {
    pub height: i16,
    pub saved_height: i16,
    pub saved_region: u32,
    pub desktop_region: u32,
}

/// hideMenuBar, 0x14ee9e. Both new regions are retained by the native body.
pub fn hide_menu_bar(
    context: &mut MenuBarContext,
    services: &mut impl InterfaceServices,
) -> Result<InterfacePlan, RuleError> {
    let mut candidate = context.clone();
    let mut effects = Vec::new();
    candidate.saved_height = candidate.height;
    write(
        &mut effects,
        0x4042c2,
        &candidate.saved_height.to_be_bytes(),
    );
    candidate.height = 0;
    write(&mut effects, 0xbaa, &[0, 0]);
    effects.push(InterfaceEffect::NewRegion);
    candidate.saved_region = services.new_region()?;
    write(
        &mut effects,
        0x4042c4,
        &candidate.saved_region.to_be_bytes(),
    );
    effects.push(InterfaceEffect::CopyRegion {
        source: candidate.desktop_region,
        destination: candidate.saved_region,
    });
    effects.push(InterfaceEffect::NewRegion);
    let region = services.new_region()?;
    effects.push(InterfaceEffect::GetMainDevice);
    let rect = services.main_device_rect()?;
    effects.push(InterfaceEffect::RectRegion { region, rect });
    effects.push(InterfaceEffect::UnionRegion {
        source_a: candidate.saved_region,
        source_b: region,
        destination: candidate.desktop_region,
    });
    effects.push(InterfaceEffect::PaintOne {
        window: 0,
        region: candidate.saved_region,
    });
    *context = candidate;
    Ok(InterfacePlan {
        native_return: 0,
        effects,
    })
}

/// showMenuBar, 0x14ef36: height restore precedes UnionRgn and DrawMenuBar.
pub fn show_menu_bar(context: &mut MenuBarContext) -> InterfacePlan {
    context.height = context.saved_height;
    let mut effects = Vec::new();
    write(&mut effects, 0xbaa, &context.height.to_be_bytes());
    effects.push(InterfaceEffect::SectRegion {
        source_a: context.saved_region,
        source_b: context.desktop_region,
        destination: context.desktop_region,
    });
    effects.push(InterfaceEffect::DrawMenuBar);
    InterfacePlan {
        native_return: 0,
        effects,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightInput {
    pub member: i8,
    pub override_picture: i8,
    pub port: u32,
    pub width_input: i16,
    pub threshold: i16,
    pub ordinary_picture: u32,
    pub override_handle: u32,
    pub low_picture: u32,
}

/// updatelight, 0x160fb2: MULS positions, but MULU selects the warrior record.
/// The native void return is represented as effects without an invented D0.
pub fn updatelight(
    input: LightInput,
    services: &mut impl InterfaceServices,
) -> Result<Vec<InterfaceEffect>, RuleError> {
    let top = i16::from(input.member).wrapping_mul(50).wrapping_add(9);
    let left = input.width_input.wrapping_add(387);
    let rect = BodyRect {
        top,
        left,
        bottom: top.wrapping_add(9),
        right: left.wrapping_add(9),
    };
    let handle = if input.override_picture != 0 {
        input.override_handle
    } else {
        let address = 0x448aa0 + u32::from(i16::from(input.member) as u16) * 0xc42 + 0x20;
        let first = services.word(address)?;
        if input.threshold > first {
            input.low_picture
        } else {
            // The second read is reached only after the first signed comparison.
            let second = services.word(address)?;
            if second == -1 {
                input.low_picture
            } else {
                input.ordinary_picture
            }
        }
    };
    Ok(vec![
        InterfaceEffect::SetPort(input.port),
        InterfaceEffect::DrawPicture { handle, rect },
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    struct Observations {
        words: VecDeque<i16>,
        read_addresses: Vec<u32>,
        regions: VecDeque<u32>,
        device: Option<BodyRect>,
    }
    impl InterfaceServices for Observations {
        fn word(&mut self, address: u32) -> Result<i16, RuleError> {
            self.read_addresses.push(address);
            self.words
                .pop_front()
                .ok_or_else(|| RuleError::Invalid("missing test word".into()))
        }
        fn current_port(&mut self) -> Result<u32, RuleError> {
            Ok(80)
        }
        fn pen_state(&mut self) -> Result<[u8; 18], RuleError> {
            Ok([7; 18])
        }
        fn local_to_global(&mut self, point: Point) -> Result<Point, RuleError> {
            assert_eq!(
                point,
                Point {
                    vertical: 176,
                    horizontal: 176
                }
            );
            Ok(Point {
                vertical: -1,
                horizontal: 32767,
            })
        }
        fn point_in_rect(&mut self, point: Point, rect: BodyRect) -> Result<u8, RuleError> {
            assert_eq!(
                point,
                Point {
                    vertical: 5,
                    horizontal: 4
                }
            );
            assert_eq!(
                rect,
                BodyRect {
                    top: 1,
                    left: 1,
                    bottom: -32760,
                    right: -24568
                }
            );
            Ok(255)
        }
        fn new_region(&mut self) -> Result<u32, RuleError> {
            self.regions
                .pop_front()
                .ok_or_else(|| RuleError::Invalid("missing test region".into()))
        }
        fn main_device_rect(&mut self) -> Result<BodyRect, RuleError> {
            self.device
                .ok_or_else(|| RuleError::Invalid("missing device bounds".into()))
        }
    }
    fn setup() -> Observations {
        Observations {
            words: [4, 5].into(),
            read_addresses: Vec::new(),
            regions: [100, 101].into(),
            device: Some(BodyRect {
                top: 0,
                left: 0,
                bottom: 640,
                right: 480,
            }),
        }
    }

    #[test]
    fn cursor_uses_transformed_point_and_body_nine_uses_negative_enemy_slot() {
        let mut services = setup();
        let plan = centercursor(&mut services).unwrap();
        assert_eq!(
            &plan.effects[1..],
            &[
                InterfaceEffect::Write(RecordWrite {
                    address: 0x82c,
                    bytes: vec![255, 255, 127, 255]
                }),
                InterfaceEffect::Write(RecordWrite {
                    address: 0x828,
                    bytes: vec![255, 255, 127, 255]
                }),
                InterfaceEffect::Write(RecordWrite {
                    address: 0x8ce,
                    bytes: vec![255, 255]
                }),
            ]
        );
        assert_eq!(inwindow(9, 8192, &mut services).unwrap().native_return, 1);
        assert_eq!(services.read_addresses, [0x446f06, 0x446f08]);
    }

    #[test]
    fn region_plans_are_atomic_and_light_keeps_unsigned_record_addressing() {
        let mut context = MenuBarContext {
            height: 20,
            saved_height: 10,
            saved_region: 50,
            desktop_region: 60,
        };
        let original = context.clone();
        let mut services = setup();
        services.device = None;
        assert!(hide_menu_bar(&mut context, &mut services).is_err());
        assert_eq!(context, original);
        services = setup();
        let hidden = hide_menu_bar(&mut context, &mut services).unwrap();
        assert_eq!(
            (context.height, context.saved_height, context.saved_region),
            (0, 20, 100)
        );
        assert_eq!(
            hidden.effects.last(),
            Some(&InterfaceEffect::PaintOne {
                window: 0,
                region: 100
            })
        );
        assert!(hidden.effects.contains(&InterfaceEffect::UnionRegion {
            source_a: 100,
            source_b: 101,
            destination: 60,
        }));
        let shown = show_menu_bar(&mut context);
        assert_eq!(context.height, 20);
        assert_eq!(
            &shown.effects[1..],
            &[
                InterfaceEffect::SectRegion {
                    source_a: 100,
                    source_b: 60,
                    destination: 60
                },
                InterfaceEffect::DrawMenuBar,
            ]
        );
        let input = LightInput {
            member: -1,
            override_picture: 0,
            port: 90,
            width_input: 32767,
            threshold: 3,
            ordinary_picture: 10,
            override_handle: 20,
            low_picture: 30,
        };
        services.words = [4, -1].into();
        let effects = updatelight(input, &mut services).unwrap();
        assert_eq!(
            services.read_addresses,
            [0x448aa0 + 65535 * 0xc42 + 0x20; 2]
        );
        assert_eq!(
            effects[1],
            InterfaceEffect::DrawPicture {
                handle: 30,
                rect: BodyRect {
                    top: -41,
                    left: -32382,
                    bottom: -32,
                    right: -32373
                }
            }
        );
        let mut override_input = input;
        override_input.override_picture = 1;
        services.words.clear();
        assert!(updatelight(override_input, &mut services).is_ok());
        assert_eq!(services.read_addresses.len(), 2);
    }
}
