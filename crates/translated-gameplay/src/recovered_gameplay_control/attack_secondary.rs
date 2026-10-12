//! Complete provisional gameplay control from canonical CODE 1 bodies.

use crate::RuleError;
use crate::recovered_application_control::{Execution, Result, Signal};
use crate::recovered_native_services::NativeCall;
use crate::recovered_system_control::{child, cpu, finish, imported};

pub fn attack_secondary_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x101eb6_u32;
        loop {
            match match pc {
                0x101eb6 => attack_secondary_body_block_0(e, &mut r, frame, pc)?,
                0x101edc => attack_secondary_body_block_1(e, &mut r, frame, pc)?,
                0x101ee2 => attack_secondary_body_block_2(e, &mut r, frame, pc)?,
                0x101efa => attack_secondary_body_block_3(e, &mut r, frame, pc)?,
                0x101f00 => attack_secondary_body_block_4(e, &mut r, frame, pc)?,
                0x101f6e => attack_secondary_body_block_5(e, &mut r, frame, pc)?,
                0x101f74 => attack_secondary_body_block_6(e, &mut r, frame, pc)?,
                0x101f82 => attack_secondary_body_block_7(e, &mut r, frame, pc)?,
                0x101f96 => attack_secondary_body_block_8(e, &mut r, frame, pc)?,
                0x101fa8 => attack_secondary_body_block_9(e, &mut r, frame, pc)?,
                0x101fb0 => attack_secondary_body_block_a(e, &mut r, frame, pc)?,
                0x101fb2 => attack_secondary_body_block_b(e, &mut r, frame, pc)?,
                0x101fbe => attack_secondary_body_block_c(e, &mut r, frame, pc)?,
                0x101fc6 => attack_secondary_body_block_d(e, &mut r, frame, pc)?,
                0x101fd6 => attack_secondary_body_block_e(e, &mut r, frame, pc)?,
                0x101fde => attack_secondary_body_block_f(e, &mut r, frame, pc)?,
                0x101fe4 => attack_secondary_body_block_10(e, &mut r, frame, pc)?,
                0x101ff2 => attack_secondary_body_block_11(e, &mut r, frame, pc)?,
                0x10201a => attack_secondary_body_block_12(e, &mut r, frame, pc)?,
                0x102024 => attack_secondary_body_block_13(e, &mut r, frame, pc)?,
                0x10203e => attack_secondary_body_block_14(e, &mut r, frame, pc)?,
                0x102052 => attack_secondary_body_block_15(e, &mut r, frame, pc)?,
                0x10205c => attack_secondary_body_block_16(e, &mut r, frame, pc)?,
                0x102070 => attack_secondary_body_block_17(e, &mut r, frame, pc)?,
                0x102072 => attack_secondary_body_block_18(e, &mut r, frame, pc)?,
                0x10207c => attack_secondary_body_block_19(e, &mut r, frame, pc)?,
                0x102090 => attack_secondary_body_block_1a(e, &mut r, frame, pc)?,
                0x1020a8 => attack_secondary_body_block_1b(e, &mut r, frame, pc)?,
                0x1020b4 => attack_secondary_body_block_1c(e, &mut r, frame, pc)?,
                0x1020ce => attack_secondary_body_block_1d(e, &mut r, frame, pc)?,
                0x1020e0 => attack_secondary_body_block_1e(e, &mut r, frame, pc)?,
                0x1020e6 => attack_secondary_body_block_1f(e, &mut r, frame, pc)?,
                0x1020ea => attack_secondary_body_block_20(e, &mut r, frame, pc)?,
                0x1020f0 => attack_secondary_body_block_21(e, &mut r, frame, pc)?,
                0x1020f4 => attack_secondary_body_block_22(e, &mut r, frame, pc)?,
                0x1020fa => attack_secondary_body_block_23(e, &mut r, frame, pc)?,
                0x1020fe => attack_secondary_body_block_24(e, &mut r, frame, pc)?,
                0x102104 => attack_secondary_body_block_25(e, &mut r, frame, pc)?,
                0x102116 => attack_secondary_body_block_26(e, &mut r, frame, pc)?,
                0x10211e => attack_secondary_body_block_27(e, &mut r, frame, pc)?,
                0x102138 => attack_secondary_body_block_28(e, &mut r, frame, pc)?,
                0x10213e => attack_secondary_body_block_29(e, &mut r, frame, pc)?,
                0x10216e => attack_secondary_body_block_2a(e, &mut r, frame, pc)?,
                0x102188 => attack_secondary_body_block_2b(e, &mut r, frame, pc)?,
                0x102190 => attack_secondary_body_block_2c(e, &mut r, frame, pc)?,
                0x1021bc => attack_secondary_body_block_2d(e, &mut r, frame, pc)?,
                0x1021c2 => attack_secondary_body_block_2e(e, &mut r, frame, pc)?,
                0x1021d0 => attack_secondary_body_block_2f(e, &mut r, frame, pc)?,
                0x1021d6 => attack_secondary_body_block_30(e, &mut r, frame, pc)?,
                0x1021da => attack_secondary_body_block_31(e, &mut r, frame, pc)?,
                0x1021e0 => attack_secondary_body_block_32(e, &mut r, frame, pc)?,
                0x1021e4 => attack_secondary_body_block_33(e, &mut r, frame, pc)?,
                0x1021ea => attack_secondary_body_block_34(e, &mut r, frame, pc)?,
                0x1021ee => attack_secondary_body_block_35(e, &mut r, frame, pc)?,
                0x1021f4 => attack_secondary_body_block_36(e, &mut r, frame, pc)?,
                0x102202 => attack_secondary_body_block_37(e, &mut r, frame, pc)?,
                0x102208 => attack_secondary_body_block_38(e, &mut r, frame, pc)?,
                0x102216 => attack_secondary_body_block_39(e, &mut r, frame, pc)?,
                0x102220 => attack_secondary_body_block_3a(e, &mut r, frame, pc)?,
                0x102228 => attack_secondary_body_block_3b(e, &mut r, frame, pc)?,
                0x102230 => attack_secondary_body_block_3c(e, &mut r, frame, pc)?,
                0x102238 => attack_secondary_body_block_3d(e, &mut r, frame, pc)?,
                0x102256 => attack_secondary_body_block_3e(e, &mut r, frame, pc)?,
                0x10225c => attack_secondary_body_block_3f(e, &mut r, frame, pc)?,
                0x10228c => attack_secondary_body_block_40(e, &mut r, frame, pc)?,
                0x1022aa => attack_secondary_body_block_41(e, &mut r, frame, pc)?,
                0x1022b2 => attack_secondary_body_block_42(e, &mut r, frame, pc)?,
                0x1022de => attack_secondary_body_block_43(e, &mut r, frame, pc)?,
                0x1022f4 => attack_secondary_body_block_44(e, &mut r, frame, pc)?,
                0x102316 => attack_secondary_body_block_45(e, &mut r, frame, pc)?,
                0x10232c => attack_secondary_body_block_46(e, &mut r, frame, pc)?,
                0x102330 => attack_secondary_body_block_47(e, &mut r, frame, pc)?,
                0x102346 => attack_secondary_body_block_48(e, &mut r, frame, pc)?,
                0x10234a => attack_secondary_body_block_49(e, &mut r, frame, pc)?,
                0x102360 => attack_secondary_body_block_4a(e, &mut r, frame, pc)?,
                0x102364 => attack_secondary_body_block_4b(e, &mut r, frame, pc)?,
                0x10237a => attack_secondary_body_block_4c(e, &mut r, frame, pc)?,
                0x10239c => attack_secondary_body_block_4d(e, &mut r, frame, pc)?,
                0x1023b2 => attack_secondary_body_block_4e(e, &mut r, frame, pc)?,
                0x1023d4 => attack_secondary_body_block_4f(e, &mut r, frame, pc)?,
                0x1023ec => attack_secondary_body_block_50(e, &mut r, frame, pc)?,
                0x1023f2 => attack_secondary_body_block_51(e, &mut r, frame, pc)?,
                0x102408 => attack_secondary_body_block_52(e, &mut r, frame, pc)?,
                0x10240c => attack_secondary_body_block_53(e, &mut r, frame, pc)?,
                0x102442 => attack_secondary_body_block_54(e, &mut r, frame, pc)?,
                0x102448 => attack_secondary_body_block_55(e, &mut r, frame, pc)?,
                0x102450 => attack_secondary_body_block_56(e, &mut r, frame, pc)?,
                0x102458 => attack_secondary_body_block_57(e, &mut r, frame, pc)?,
                0x10246c => attack_secondary_body_block_58(e, &mut r, frame, pc)?,
                0x102482 => attack_secondary_body_block_59(e, &mut r, frame, pc)?,
                0x102498 => attack_secondary_body_block_5a(e, &mut r, frame, pc)?,
                0x1024b6 => attack_secondary_body_block_5b(e, &mut r, frame, pc)?,
                0x1024d8 => attack_secondary_body_block_5c(e, &mut r, frame, pc)?,
                0x1024e0 => attack_secondary_body_block_5d(e, &mut r, frame, pc)?,
                0x1024e8 => attack_secondary_body_block_5e(e, &mut r, frame, pc)?,
                0x1024fe => attack_secondary_body_block_5f(e, &mut r, frame, pc)?,
                0x102508 => attack_secondary_body_block_60(e, &mut r, frame, pc)?,
                0x10250e => attack_secondary_body_block_61(e, &mut r, frame, pc)?,
                0x102530 => attack_secondary_body_block_62(e, &mut r, frame, pc)?,
                0x102568 => attack_secondary_body_block_63(e, &mut r, frame, pc)?,
                0x102570 => attack_secondary_body_block_64(e, &mut r, frame, pc)?,
                0x102578 => attack_secondary_body_block_65(e, &mut r, frame, pc)?,
                0x10258e => attack_secondary_body_block_66(e, &mut r, frame, pc)?,
                0x102598 => attack_secondary_body_block_67(e, &mut r, frame, pc)?,
                0x10259e => attack_secondary_body_block_68(e, &mut r, frame, pc)?,
                0x1025c0 => attack_secondary_body_block_69(e, &mut r, frame, pc)?,
                0x1025f8 => attack_secondary_body_block_6a(e, &mut r, frame, pc)?,
                0x1025fe => attack_secondary_body_block_6b(e, &mut r, frame, pc)?,
                0x102604 => attack_secondary_body_block_6c(e, &mut r, frame, pc)?,
                0x10260a => attack_secondary_body_block_6d(e, &mut r, frame, pc)?,
                0x102620 => attack_secondary_body_block_6e(e, &mut r, frame, pc)?,
                0x102634 => attack_secondary_body_block_6f(e, &mut r, frame, pc)?,
                0x10264a => attack_secondary_body_block_70(e, &mut r, frame, pc)?,
                0x102650 => attack_secondary_body_block_71(e, &mut r, frame, pc)?,
                0x102656 => attack_secondary_body_block_72(e, &mut r, frame, pc)?,
                0x10265c => attack_secondary_body_block_73(e, &mut r, frame, pc)?,
                0x102674 => attack_secondary_body_block_74(e, &mut r, frame, pc)?,
                0x102686 => attack_secondary_body_block_75(e, &mut r, frame, pc)?,
                0x10269a => attack_secondary_body_block_76(e, &mut r, frame, pc)?,
                0x10269e => attack_secondary_body_block_77(e, &mut r, frame, pc)?,
                0x1026a4 => attack_secondary_body_block_78(e, &mut r, frame, pc)?,
                0x1026aa => attack_secondary_body_block_79(e, &mut r, frame, pc)?,
                0x1026c4 => attack_secondary_body_block_7a(e, &mut r, frame, pc)?,
                0x1026ca => attack_secondary_body_block_7b(e, &mut r, frame, pc)?,
                0x1026e0 => attack_secondary_body_block_7c(e, &mut r, frame, pc)?,
                0x1026fc => attack_secondary_body_block_7d(e, &mut r, frame, pc)?,
                0x102704 => attack_secondary_body_block_7e(e, &mut r, frame, pc)?,
                0x10270a => attack_secondary_body_block_7f(e, &mut r, frame, pc)?,
                0x102720 => attack_secondary_body_block_80(e, &mut r, frame, pc)?,
                0x102746 => attack_secondary_body_block_81(e, &mut r, frame, pc)?,
                0x10275a => attack_secondary_body_block_82(e, &mut r, frame, pc)?,
                0x1027b2 => attack_secondary_body_block_83(e, &mut r, frame, pc)?,
                0x1027be => attack_secondary_body_block_84(e, &mut r, frame, pc)?,
                0x1027ec => attack_secondary_body_block_85(e, &mut r, frame, pc)?,
                0x1027f8 => attack_secondary_body_block_86(e, &mut r, frame, pc)?,
                0x102812 => attack_secondary_body_block_87(e, &mut r, frame, pc)?,
                0x10281c => attack_secondary_body_block_88(e, &mut r, frame, pc)?,
                0x102824 => attack_secondary_body_block_89(e, &mut r, frame, pc)?,
                0x10282e => attack_secondary_body_block_8a(e, &mut r, frame, pc)?,
                0x102836 => attack_secondary_body_block_8b(e, &mut r, frame, pc)?,
                0x102840 => attack_secondary_body_block_8c(e, &mut r, frame, pc)?,
                0x102848 => attack_secondary_body_block_8d(e, &mut r, frame, pc)?,
                0x102852 => attack_secondary_body_block_8e(e, &mut r, frame, pc)?,
                0x10285a => attack_secondary_body_block_8f(e, &mut r, frame, pc)?,
                0x102864 => attack_secondary_body_block_90(e, &mut r, frame, pc)?,
                0x10286c => attack_secondary_body_block_91(e, &mut r, frame, pc)?,
                0x102876 => attack_secondary_body_block_92(e, &mut r, frame, pc)?,
                0x10287e => attack_secondary_body_block_93(e, &mut r, frame, pc)?,
                0x102888 => attack_secondary_body_block_94(e, &mut r, frame, pc)?,
                0x1028a4 => attack_secondary_body_block_95(e, &mut r, frame, pc)?,
                0x1028be => attack_secondary_body_block_96(e, &mut r, frame, pc)?,
                0x1028c6 => attack_secondary_body_block_97(e, &mut r, frame, pc)?,
                0x1028f2 => attack_secondary_body_block_98(e, &mut r, frame, pc)?,
                0x1028fa => attack_secondary_body_block_99(e, &mut r, frame, pc)?,
                0x102902 => attack_secondary_body_block_9a(e, &mut r, frame, pc)?,
                0x10290a => attack_secondary_body_block_9b(e, &mut r, frame, pc)?,
                0x102914 => attack_secondary_body_block_9c(e, &mut r, frame, pc)?,
                0x102944 => attack_secondary_body_block_9d(e, &mut r, frame, pc)?,
                0x102956 => attack_secondary_body_block_9e(e, &mut r, frame, pc)?,
                0x102960 => attack_secondary_body_block_9f(e, &mut r, frame, pc)?,
                0x102974 => attack_secondary_body_block_a0(e, &mut r, frame, pc)?,
                0x10297a => attack_secondary_body_block_a1(e, &mut r, frame, pc)?,
                0x10299e => attack_secondary_body_block_a2(e, &mut r, frame, pc)?,
                0x1029be => attack_secondary_body_block_a3(e, &mut r, frame, pc)?,
                0x1029d2 => attack_secondary_body_block_a4(e, &mut r, frame, pc)?,
                0x1029e6 => attack_secondary_body_block_a5(e, &mut r, frame, pc)?,
                0x1029f0 => attack_secondary_body_block_a6(e, &mut r, frame, pc)?,
                0x102a04 => attack_secondary_body_block_a7(e, &mut r, frame, pc)?,
                0x102a0a => attack_secondary_body_block_a8(e, &mut r, frame, pc)?,
                0x102a2e => attack_secondary_body_block_a9(e, &mut r, frame, pc)?,
                0x102a4e => attack_secondary_body_block_aa(e, &mut r, frame, pc)?,
                0x102a82 => attack_secondary_body_block_ab(e, &mut r, frame, pc)?,
                0x102a96 => attack_secondary_body_block_ac(e, &mut r, frame, pc)?,
                0x102aa0 => attack_secondary_body_block_ad(e, &mut r, frame, pc)?,
                0x102ac4 => attack_secondary_body_block_ae(e, &mut r, frame, pc)?,
                0x102ae4 => attack_secondary_body_block_af(e, &mut r, frame, pc)?,
                0x102af8 => attack_secondary_body_block_b0(e, &mut r, frame, pc)?,
                0x102b0c => attack_secondary_body_block_b1(e, &mut r, frame, pc)?,
                0x102b16 => attack_secondary_body_block_b2(e, &mut r, frame, pc)?,
                0x102b3a => attack_secondary_body_block_b3(e, &mut r, frame, pc)?,
                0x102b5a => attack_secondary_body_block_b4(e, &mut r, frame, pc)?,
                0x102ba2 => attack_secondary_body_block_b5(e, &mut r, frame, pc)?,
                0x102bb6 => attack_secondary_body_block_b6(e, &mut r, frame, pc)?,
                0x102bc0 => attack_secondary_body_block_b7(e, &mut r, frame, pc)?,
                0x102bd4 => attack_secondary_body_block_b8(e, &mut r, frame, pc)?,
                0x102bda => attack_secondary_body_block_b9(e, &mut r, frame, pc)?,
                0x102bfe => attack_secondary_body_block_ba(e, &mut r, frame, pc)?,
                0x102c1e => attack_secondary_body_block_bb(e, &mut r, frame, pc)?,
                0x102c32 => attack_secondary_body_block_bc(e, &mut r, frame, pc)?,
                0x102c44 => attack_secondary_body_block_bd(e, &mut r, frame, pc)?,
                0x102c4e => attack_secondary_body_block_be(e, &mut r, frame, pc)?,
                0x102c62 => attack_secondary_body_block_bf(e, &mut r, frame, pc)?,
                0x102c68 => attack_secondary_body_block_c0(e, &mut r, frame, pc)?,
                0x102c8e => attack_secondary_body_block_c1(e, &mut r, frame, pc)?,
                0x102cae => attack_secondary_body_block_c2(e, &mut r, frame, pc)?,
                0x102cc2 => attack_secondary_body_block_c3(e, &mut r, frame, pc)?,
                0x102cd6 => attack_secondary_body_block_c4(e, &mut r, frame, pc)?,
                0x102cea => attack_secondary_body_block_c5(e, &mut r, frame, pc)?,
                0x102d0e => attack_secondary_body_block_c6(e, &mut r, frame, pc)?,
                0x102d22 => attack_secondary_body_block_c7(e, &mut r, frame, pc)?,
                0x102d68 => attack_secondary_body_block_c8(e, &mut r, frame, pc)?,
                0x102d7c => attack_secondary_body_block_c9(e, &mut r, frame, pc)?,
                0x102dc4 => attack_secondary_body_block_ca(e, &mut r, frame, pc)?,
                0x102dd8 => attack_secondary_body_block_cb(e, &mut r, frame, pc)?,
                0x102de2 => attack_secondary_body_block_cc(e, &mut r, frame, pc)?,
                0x102df6 => attack_secondary_body_block_cd(e, &mut r, frame, pc)?,
                0x102dfc => attack_secondary_body_block_ce(e, &mut r, frame, pc)?,
                0x102e22 => attack_secondary_body_block_cf(e, &mut r, frame, pc)?,
                0x102e42 => attack_secondary_body_block_d0(e, &mut r, frame, pc)?,
                0x102e56 => attack_secondary_body_block_d1(e, &mut r, frame, pc)?,
                0x102e68 => attack_secondary_body_block_d2(e, &mut r, frame, pc)?,
                0x102e7e => attack_secondary_body_block_d3(e, &mut r, frame, pc)?,
                0x102e90 => attack_secondary_body_block_d4(e, &mut r, frame, pc)?,
                0x102eac => attack_secondary_body_block_d5(e, &mut r, frame, pc)?,
                0x102ec2 => attack_secondary_body_block_d6(e, &mut r, frame, pc)?,
                0x102ef8 => attack_secondary_body_block_d7(e, &mut r, frame, pc)?,
                0x102f0a => attack_secondary_body_block_d8(e, &mut r, frame, pc)?,
                0x102f1c => attack_secondary_body_block_d9(e, &mut r, frame, pc)?,
                0x102f2e => attack_secondary_body_block_da(e, &mut r, frame, pc)?,
                0x102f3e => attack_secondary_body_block_db(e, &mut r, frame, pc)?,
                0x102f52 => attack_secondary_body_block_dc(e, &mut r, frame, pc)?,
                0x102f88 => attack_secondary_body_block_dd(e, &mut r, frame, pc)?,
                0x102f9a => attack_secondary_body_block_de(e, &mut r, frame, pc)?,
                0x102fac => attack_secondary_body_block_df(e, &mut r, frame, pc)?,
                0x102fbe => attack_secondary_body_block_e0(e, &mut r, frame, pc)?,
                0x102fce => attack_secondary_body_block_e1(e, &mut r, frame, pc)?,
                0x102fe2 => attack_secondary_body_block_e2(e, &mut r, frame, pc)?,
                0x103018 => attack_secondary_body_block_e3(e, &mut r, frame, pc)?,
                0x10302a => attack_secondary_body_block_e4(e, &mut r, frame, pc)?,
                0x10303c => attack_secondary_body_block_e5(e, &mut r, frame, pc)?,
                0x10304e => attack_secondary_body_block_e6(e, &mut r, frame, pc)?,
                0x10305e => attack_secondary_body_block_e7(e, &mut r, frame, pc)?,
                0x103072 => attack_secondary_body_block_e8(e, &mut r, frame, pc)?,
                0x1030ac => attack_secondary_body_block_e9(e, &mut r, frame, pc)?,
                0x1030ba => attack_secondary_body_block_ea(e, &mut r, frame, pc)?,
                0x1030cc => attack_secondary_body_block_eb(e, &mut r, frame, pc)?,
                0x1030de => attack_secondary_body_block_ec(e, &mut r, frame, pc)?,
                0x1030f0 => attack_secondary_body_block_ed(e, &mut r, frame, pc)?,
                0x103102 => attack_secondary_body_block_ee(e, &mut r, frame, pc)?,
                0x103112 => attack_secondary_body_block_ef(e, &mut r, frame, pc)?,
                0x103126 => attack_secondary_body_block_f0(e, &mut r, frame, pc)?,
                0x10315c => attack_secondary_body_block_f1(e, &mut r, frame, pc)?,
                0x10316e => attack_secondary_body_block_f2(e, &mut r, frame, pc)?,
                0x103180 => attack_secondary_body_block_f3(e, &mut r, frame, pc)?,
                0x103192 => attack_secondary_body_block_f4(e, &mut r, frame, pc)?,
                0x1031a2 => attack_secondary_body_block_f5(e, &mut r, frame, pc)?,
                0x1031b6 => attack_secondary_body_block_f6(e, &mut r, frame, pc)?,
                0x1031ec => attack_secondary_body_block_f7(e, &mut r, frame, pc)?,
                0x1031fe => attack_secondary_body_block_f8(e, &mut r, frame, pc)?,
                0x103210 => attack_secondary_body_block_f9(e, &mut r, frame, pc)?,
                0x103222 => attack_secondary_body_block_fa(e, &mut r, frame, pc)?,
                0x103232 => attack_secondary_body_block_fb(e, &mut r, frame, pc)?,
                0x103246 => attack_secondary_body_block_fc(e, &mut r, frame, pc)?,
                0x103280 => attack_secondary_body_block_fd(e, &mut r, frame, pc)?,
                0x103286 => attack_secondary_body_block_fe(e, &mut r, frame, pc)?,
                0x103298 => attack_secondary_body_block_ff(e, &mut r, frame, pc)?,
                0x1032aa => attack_secondary_body_block_100(e, &mut r, frame, pc)?,
                0x1032bc => attack_secondary_body_block_101(e, &mut r, frame, pc)?,
                0x1032ce => attack_secondary_body_block_102(e, &mut r, frame, pc)?,
                0x1032de => attack_secondary_body_block_103(e, &mut r, frame, pc)?,
                0x1032f2 => attack_secondary_body_block_104(e, &mut r, frame, pc)?,
                0x103306 => attack_secondary_body_block_105(e, &mut r, frame, pc)?,
                0x10331c => attack_secondary_body_block_106(e, &mut r, frame, pc)?,
                0x103344 => attack_secondary_body_block_107(e, &mut r, frame, pc)?,
                0x10335c => attack_secondary_body_block_108(e, &mut r, frame, pc)?,
                0x103366 => attack_secondary_body_block_109(e, &mut r, frame, pc)?,
                0x103396 => attack_secondary_body_block_10a(e, &mut r, frame, pc)?,
                0x1033a8 => attack_secondary_body_block_10b(e, &mut r, frame, pc)?,
                0x1033c2 => attack_secondary_body_block_10c(e, &mut r, frame, pc)?,
                0x1033f6 => attack_secondary_body_block_10d(e, &mut r, frame, pc)?,
                0x10340a => attack_secondary_body_block_10e(e, &mut r, frame, pc)?,
                0x103424 => attack_secondary_body_block_10f(e, &mut r, frame, pc)?,
                0x103480 => attack_secondary_body_block_110(e, &mut r, frame, pc)?,
                0x103494 => attack_secondary_body_block_111(e, &mut r, frame, pc)?,
                0x1034ae => attack_secondary_body_block_112(e, &mut r, frame, pc)?,
                0x1034e2 => attack_secondary_body_block_113(e, &mut r, frame, pc)?,
                0x1034f6 => attack_secondary_body_block_114(e, &mut r, frame, pc)?,
                0x103510 => attack_secondary_body_block_115(e, &mut r, frame, pc)?,
                0x103580 => attack_secondary_body_block_116(e, &mut r, frame, pc)?,
                0x103594 => attack_secondary_body_block_117(e, &mut r, frame, pc)?,
                0x1035ae => attack_secondary_body_block_118(e, &mut r, frame, pc)?,
                0x1035e2 => attack_secondary_body_block_119(e, &mut r, frame, pc)?,
                0x1035f4 => attack_secondary_body_block_11a(e, &mut r, frame, pc)?,
                0x10360e => attack_secondary_body_block_11b(e, &mut r, frame, pc)?,
                0x103642 => attack_secondary_body_block_11c(e, &mut r, frame, pc)?,
                0x103656 => attack_secondary_body_block_11d(e, &mut r, frame, pc)?,
                0x10366e => attack_secondary_body_block_11e(e, &mut r, frame, pc)?,
                0x103696 => attack_secondary_body_block_11f(e, &mut r, frame, pc)?,
                0x1036ae => attack_secondary_body_block_120(e, &mut r, frame, pc)?,
                0x1036f6 => attack_secondary_body_block_121(e, &mut r, frame, pc)?,
                0x10370a => attack_secondary_body_block_122(e, &mut r, frame, pc)?,
                0x103724 => attack_secondary_body_block_123(e, &mut r, frame, pc)?,
                0x103758 => attack_secondary_body_block_124(e, &mut r, frame, pc)?,
                0x10376a => attack_secondary_body_block_125(e, &mut r, frame, pc)?,
                0x103770 => attack_secondary_body_block_126(e, &mut r, frame, pc)?,
                0x103776 => attack_secondary_body_block_127(e, &mut r, frame, pc)?,
                0x10377c => attack_secondary_body_block_128(e, &mut r, frame, pc)?,
                0x103790 => attack_secondary_body_block_129(e, &mut r, frame, pc)?,
                0x103796 => attack_secondary_body_block_12a(e, &mut r, frame, pc)?,
                0x1037d4 => attack_secondary_body_block_12b(e, &mut r, frame, pc)?,
                0x10380a => attack_secondary_body_block_12c(e, &mut r, frame, pc)?,
                0x10381c => attack_secondary_body_block_12d(e, &mut r, frame, pc)?,
                0x103832 => attack_secondary_body_block_12e(e, &mut r, frame, pc)?,
                0x103844 => attack_secondary_body_block_12f(e, &mut r, frame, pc)?,
                0x103854 => attack_secondary_body_block_130(e, &mut r, frame, pc)?,
                0x103868 => attack_secondary_body_block_131(e, &mut r, frame, pc)?,
                0x10389e => attack_secondary_body_block_132(e, &mut r, frame, pc)?,
                0x1038b0 => attack_secondary_body_block_133(e, &mut r, frame, pc)?,
                0x1038c6 => attack_secondary_body_block_134(e, &mut r, frame, pc)?,
                0x1038d8 => attack_secondary_body_block_135(e, &mut r, frame, pc)?,
                0x1038e8 => attack_secondary_body_block_136(e, &mut r, frame, pc)?,
                0x1038fc => attack_secondary_body_block_137(e, &mut r, frame, pc)?,
                0x103932 => attack_secondary_body_block_138(e, &mut r, frame, pc)?,
                0x103944 => attack_secondary_body_block_139(e, &mut r, frame, pc)?,
                0x10395a => attack_secondary_body_block_13a(e, &mut r, frame, pc)?,
                0x10396c => attack_secondary_body_block_13b(e, &mut r, frame, pc)?,
                0x10397c => attack_secondary_body_block_13c(e, &mut r, frame, pc)?,
                0x103990 => attack_secondary_body_block_13d(e, &mut r, frame, pc)?,
                0x1039c6 => attack_secondary_body_block_13e(e, &mut r, frame, pc)?,
                0x1039d8 => attack_secondary_body_block_13f(e, &mut r, frame, pc)?,
                0x1039ee => attack_secondary_body_block_140(e, &mut r, frame, pc)?,
                0x103a00 => attack_secondary_body_block_141(e, &mut r, frame, pc)?,
                0x103a10 => attack_secondary_body_block_142(e, &mut r, frame, pc)?,
                0x103a24 => attack_secondary_body_block_143(e, &mut r, frame, pc)?,
                0x103a5a => attack_secondary_body_block_144(e, &mut r, frame, pc)?,
                0x103a6c => attack_secondary_body_block_145(e, &mut r, frame, pc)?,
                0x103a82 => attack_secondary_body_block_146(e, &mut r, frame, pc)?,
                0x103a94 => attack_secondary_body_block_147(e, &mut r, frame, pc)?,
                0x103aa4 => attack_secondary_body_block_148(e, &mut r, frame, pc)?,
                0x103ab8 => attack_secondary_body_block_149(e, &mut r, frame, pc)?,
                0x103aee => attack_secondary_body_block_14a(e, &mut r, frame, pc)?,
                0x103b00 => attack_secondary_body_block_14b(e, &mut r, frame, pc)?,
                0x103b16 => attack_secondary_body_block_14c(e, &mut r, frame, pc)?,
                0x103b28 => attack_secondary_body_block_14d(e, &mut r, frame, pc)?,
                0x103b38 => attack_secondary_body_block_14e(e, &mut r, frame, pc)?,
                0x103b4c => attack_secondary_body_block_14f(e, &mut r, frame, pc)?,
                0x103b82 => attack_secondary_body_block_150(e, &mut r, frame, pc)?,
                0x103b94 => attack_secondary_body_block_151(e, &mut r, frame, pc)?,
                0x103baa => attack_secondary_body_block_152(e, &mut r, frame, pc)?,
                0x103bbc => attack_secondary_body_block_153(e, &mut r, frame, pc)?,
                0x103bca => attack_secondary_body_block_154(e, &mut r, frame, pc)?,
                0x103bdc => attack_secondary_body_block_155(e, &mut r, frame, pc)?,
                0x103bee => attack_secondary_body_block_156(e, &mut r, frame, pc)?,
                0x103c14 => attack_secondary_body_block_157(e, &mut r, frame, pc)?,
                0x103c1c => attack_secondary_body_block_158(e, &mut r, frame, pc)?,
                0x103c26 => attack_secondary_body_block_159(e, &mut r, frame, pc)?,
                0x103c2c => attack_secondary_body_block_15a(e, &mut r, frame, pc)?,
                0x103c3c => attack_secondary_body_block_15b(e, &mut r, frame, pc)?,
                0x103c44 => attack_secondary_body_block_15c(e, &mut r, frame, pc)?,
                0x103c50 => attack_secondary_body_block_15d(e, &mut r, frame, pc)?,
                0x103c58 => attack_secondary_body_block_15e(e, &mut r, frame, pc)?,
                0x103c5e => attack_secondary_body_block_15f(e, &mut r, frame, pc)?,
                0x103c7a => attack_secondary_body_block_160(e, &mut r, frame, pc)?,
                0x103c98 => attack_secondary_body_block_161(e, &mut r, frame, pc)?,
                0x103ca8 => attack_secondary_body_block_162(e, &mut r, frame, pc)?,
                0x103d14 => attack_secondary_body_block_163(e, &mut r, frame, pc)?,
                0x103d28 => attack_secondary_body_block_164(e, &mut r, frame, pc)?,
                0x103d3c => attack_secondary_body_block_165(e, &mut r, frame, pc)?,
                0x103d52 => attack_secondary_body_block_166(e, &mut r, frame, pc)?,
                0x103d58 => attack_secondary_body_block_167(e, &mut r, frame, pc)?,
                0x103d78 => attack_secondary_body_block_168(e, &mut r, frame, pc)?,
                0x103d98 => attack_secondary_body_block_169(e, &mut r, frame, pc)?,
                0x103da8 => attack_secondary_body_block_16a(e, &mut r, frame, pc)?,
                0x103dec => attack_secondary_body_block_16b(e, &mut r, frame, pc)?,
                0x103dfc => attack_secondary_body_block_16c(e, &mut r, frame, pc)?,
                0x103e02 => attack_secondary_body_block_16d(e, &mut r, frame, pc)?,
                0x103e14 => attack_secondary_body_block_16e(e, &mut r, frame, pc)?,
                0x103e36 => attack_secondary_body_block_16f(e, &mut r, frame, pc)?,
                0x103e3c => attack_secondary_body_block_170(e, &mut r, frame, pc)?,
                0x103e50 => attack_secondary_body_block_171(e, &mut r, frame, pc)?,
                0x103e64 => attack_secondary_body_block_172(e, &mut r, frame, pc)?,
                0x103e7c => attack_secondary_body_block_173(e, &mut r, frame, pc)?,
                0x103e8e => attack_secondary_body_block_174(e, &mut r, frame, pc)?,
                0x103eac => attack_secondary_body_block_175(e, &mut r, frame, pc)?,
                _ => attack_secondary_body_block_176(e, &mut r, frame, pc)?,
            } {
                std::ops::ControlFlow::Continue(next) => pc = next,
                std::ops::ControlFlow::Break(()) => return Ok(()),
            }
        }
    })();
    result.map_err(|signal| match signal {
        Signal::Rule(error) => Signal::Rule(r.opaque_error(error)),
        stopped => stopped,
    })
}
crate::recovered_application_control::entry!(attack_secondary, 0x101eb6);
mod attack_secondary_body_block_0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101eb6: link.w a6, #$f294
        e.step()?;
        r.save_register(e, 14)?;
        r.put_address(6, r.address(7)?);
        r.offset_address(7, -3436)?;
        // 00101eba: movem.l d3-d7, -(a7)
        e.step()?;
        r.save_register(e, 7)?;
        r.save_register(e, 6)?;
        r.save_register(e, 5)?;
        r.save_register(e, 4)?;
        r.save_register(e, 3)?;
        // 00101ebe: move.w $8(a6), d4
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0x8_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(4, 16, value);
        r.move_flags(value, 16);
        // 00101ec2: move.w $a(a6), d3
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xa_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(3, 16, value);
        r.move_flags(value, 16);
        // 00101ec6: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101ec8: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101ecc: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101ed2: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101ed4: lea.l -$d62(a6), a1
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffff29e_u32);
        r.put_address(1, value);
        // 00101ed8: move.w #$30f, d0
        e.step()?;
        let value = 0x30f_u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        pc = 0x101edc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_0_unit::attack_secondary_body_block_0;

mod attack_secondary_body_block_1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101edc: move.l (a0)+, (a1)+
        e.step()?;
        let value_address = r.address(0)?;
        let value = e.l(value_address)?;
        r.offset_address(0, 4)?;
        let destination_address = r.address(1)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.offset_address(1, 4)?;
        r.move_flags(value, 32);
        // 00101ede: dbra d0, $101edc
        e.step()?;
        let counter = r.data(0, 16)?;
        let value = counter.wrapping_sub(1) & 0xffff;
        r.put_data(0, 16, value);
        pc = if value != 0xffff {
            0x101edc_u32
        } else {
            0x101ee2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_1_unit::attack_secondary_body_block_1;

mod attack_secondary_body_block_2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101ee2: move.w (a0)+, (a1)+
        e.step()?;
        let value_address = r.address(0)?;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(0, 2)?;
        let destination_address = r.address(1)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.offset_address(1, 2)?;
        r.move_flags(value, 16);
        // 00101ee4: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00101ee6: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00101ee8: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101eec: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00101ef2: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101ef4: lea.l -$120(a6), a1
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(1, value);
        // 00101ef8: moveq #$47, d0
        e.step()?;
        r.put_data(0, 32, 0x47_u32);
        r.move_flags(0x47_u32, 32);
        pc = 0x101efa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_2_unit::attack_secondary_body_block_2;

mod attack_secondary_body_block_3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101efa: move.l (a0)+, (a1)+
        e.step()?;
        let value_address = r.address(0)?;
        let value = e.l(value_address)?;
        r.offset_address(0, 4)?;
        let destination_address = r.address(1)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.offset_address(1, 4)?;
        r.move_flags(value, 32);
        // 00101efc: dbra d0, $101efa
        e.step()?;
        let counter = r.data(0, 16)?;
        let value = counter.wrapping_sub(1) & 0xffff;
        r.put_data(0, 16, value);
        pc = if value != 0xffff {
            0x101efa_u32
        } else {
            0x101f00_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_3_unit::attack_secondary_body_block_3;

mod attack_secondary_body_block_4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101f00: move.l $43879c.l, -(a7)
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00101f06: MacToolbox 0xa873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 00101f08: move.w #$e, -(a7)
        e.step()?;
        let value = 0xe_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101f0c: bsr.l $13154c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101f12_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13154c_u32)?;
        // 00101f12: clr.b -$d6b(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffff295_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00101f16: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00101f18: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00101f1a: move.w d0, -$d66(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff29a_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101f1e: move.w d0, -$d6a(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff296_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101f22: move.w d0, -$d68(a6)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff298_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101f26: move.w d0, d6
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(6, 16, value);
        r.move_flags(value, 16);
        // 00101f28: move.b d0, -$d63(a6)
        e.step()?;
        let value = r.data(0, 8)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff29d_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00101f2c: moveq #$0, d1
        e.step()?;
        r.put_data(1, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00101f2e: move.b d0, d1
        e.step()?;
        let value = r.data(0, 8)?;
        r.put_data(1, 8, value);
        r.move_flags(value, 8);
        // 00101f30: move.w d1, $4434b8.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101f36: move.w d1, $442bc8.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101f3c: move.w d1, d7
        e.step()?;
        let value = r.data(1, 16)?;
        r.put_data(7, 16, value);
        r.move_flags(value, 16);
        // 00101f3e: move.w d1, $438786.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x438786_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101f44: move.w d1, $4427a6.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x4427a6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101f4a: move.b #$1, $438582.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x438582_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00101f52: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00101f54: moveq #$1, d0
        e.step()?;
        r.put_data(0, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        // 00101f56: move.w d0, $43877e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x43877e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101f5c: lea.l $443788.l, a0
        e.step()?;
        let value = 0x443788_u32;
        r.put_address(0, value);
        // 00101f62: lea.l $21c2(a0), a0
        e.step()?;
        let value = r.address(0)?.wrapping_add(0x21c2_u32);
        r.put_address(0, value);
        // 00101f66: lea.l $44374e.l, a1
        e.step()?;
        let value = 0x44374e_u32;
        r.put_address(1, value);
        // 00101f6c: moveq #$d, d0
        e.step()?;
        r.put_data(0, 32, 0xd_u32);
        r.move_flags(0xd_u32, 32);
        pc = 0x101f6e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_4_unit::attack_secondary_body_block_4;

mod attack_secondary_body_block_5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101f6e: move.l (a0)+, (a1)+
        e.step()?;
        let value_address = r.address(0)?;
        let value = e.l(value_address)?;
        r.offset_address(0, 4)?;
        let destination_address = r.address(1)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.offset_address(1, 4)?;
        r.move_flags(value, 32);
        // 00101f70: dbra d0, $101f6e
        e.step()?;
        let counter = r.data(0, 16)?;
        let value = counter.wrapping_sub(1) & 0xffff;
        r.put_data(0, 16, value);
        pc = if value != 0xffff {
            0x101f6e_u32
        } else {
            0x101f74_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_5_unit::attack_secondary_body_block_5;

mod attack_secondary_body_block_6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101f74: move.w (a0)+, (a1)+
        e.step()?;
        let value_address = r.address(0)?;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(0, 2)?;
        let destination_address = r.address(1)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.offset_address(1, 2)?;
        r.move_flags(value, 16);
        // 00101f76: clr.w $4471ea.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4471ea_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101f7c: moveq #$0, d5
        e.step()?;
        r.put_data(5, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00101f7e: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00101f80: bra.b $101fb2
        e.step()?;
        pc = 0x101fb2_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_6_unit::attack_secondary_body_block_6;

mod attack_secondary_body_block_7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101f82: move.w d5, d0
        e.step()?;
        let value = r.data(5, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101f84: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101f88: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101f8e: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00101f90: tst.b $9df(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x9df_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00101f94: beq.b $101fb0
        e.step()?;
        pc = if r.equal()? {
            0x101fb0_u32
        } else {
            0x101f96_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_7_unit::attack_secondary_body_block_7;

mod attack_secondary_body_block_8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101f96: move.w d5, d0
        e.step()?;
        let value = r.data(5, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101f98: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00101f9c: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00101fa2: tst.w $10(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101fa6: bne.b $101fb0
        e.step()?;
        pc = if r.not_equal()? {
            0x101fb0_u32
        } else {
            0x101fa8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_8_unit::attack_secondary_body_block_8;

mod attack_secondary_body_block_9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101fa8: move.w #$1, $4471ea.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x4471ea_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x101fb0_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_9_unit::attack_secondary_body_block_9;
mod attack_secondary_body_block_a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101fb0: addq.w #$1, d5
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(5, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(5, 16, value);
        pc = 0x101fb2_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_a_unit::attack_secondary_body_block_a;

mod attack_secondary_body_block_b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101fb2: move.b $44273a.l, d0
        e.step()?;
        let value_address = 0x44273a_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00101fb8: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00101fba: cmp.w d0, d5
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(5, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00101fbc: ble.b $101f82
        e.step()?;
        pc = if r.less_equal()? {
            0x101f82_u32
        } else {
            0x101fbe_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_b_unit::attack_secondary_body_block_b;

mod attack_secondary_body_block_c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101fbe: tst.w $4471ea.l
        e.step()?;
        let value_address = 0x4471ea_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101fc4: bne.b $101fd6
        e.step()?;
        pc = if r.not_equal()? {
            0x101fd6_u32
        } else {
            0x101fc6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_c_unit::attack_secondary_body_block_c;

mod attack_secondary_body_block_d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101fc6: move.w $4435c4.l, $44374c.l
        e.step()?;
        let value_address = 0x4435c4_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x44374c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101fd0: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00101fd2: bra.w $103eac
        e.step()?;
        pc = 0x103eac_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_d_unit::attack_secondary_body_block_d;

mod attack_secondary_body_block_e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101fd6: tst.w -$d2(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff2e_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101fda: beq.w $1020a8
        e.step()?;
        pc = if r.equal()? {
            0x1020a8_u32
        } else {
            0x101fde_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_e_unit::attack_secondary_body_block_e;

mod attack_secondary_body_block_f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101fde: tst.w -$f8(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff08_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00101fe2: bne.b $101ff2
        e.step()?;
        pc = if r.not_equal()? {
            0x101ff2_u32
        } else {
            0x101fe4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_f_unit::attack_secondary_body_block_f;

mod attack_secondary_body_block_10_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_10(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101fe4: move.w -$fa(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff06_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101fe8: bsr.l $130d38
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101fee_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x130d38_u32)?;
        // 00101fee: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00101ff0: bra.b $10201a
        e.step()?;
        pc = 0x10201a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_10_unit::attack_secondary_body_block_10;

mod attack_secondary_body_block_11_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_11(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00101ff2: move.w -$f8(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff08_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00101ff6: bsr.l $130d38
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x101ffc_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x130d38_u32)?;
        // 00101ffc: move.w -$f8(a6), -$d2(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff08_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xffffff2e_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102002: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102008: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010200c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102012: move.w -$f8(a6), $4e(a0, d0.l)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff08_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r
            .address(0)?
            .wrapping_add(0x4e_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102018: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x10201a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_11_unit::attack_secondary_body_block_11;

mod attack_secondary_body_block_12_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_12(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010201a: cmpi.w #$2, $453782.l
        e.step()?;
        let source = 0x2_u32;
        let destination_address = 0x453782_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102022: beq.b $10203e
        e.step()?;
        pc = if r.equal()? {
            0x10203e_u32
        } else {
            0x102024_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_12_unit::attack_secondary_body_block_12;

mod attack_secondary_body_block_13_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_13(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102024: clr.w -$d2(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffff2e_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102028: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010202e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102032: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102038: clr.w $4e(a0, d0.l)
        e.step()?;
        let value = 0;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x4e_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010203c: bra.b $1020a8
        e.step()?;
        pc = 0x1020a8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_13_unit::attack_secondary_body_block_13;

mod attack_secondary_body_block_14_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_14(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010203e: move.b $4537b9.l, d0
        e.step()?;
        let value_address = 0x4537b9_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00102044: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102046: add.w d0, d6
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        // 00102048: cmpi.w #$fff6, $453792.l
        e.step()?;
        let source = 0xfff6_u32;
        let destination_address = 0x453792_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102050: bne.b $1020b4
        e.step()?;
        pc = if r.not_equal()? {
            0x1020b4_u32
        } else {
            0x102052_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_14_unit::attack_secondary_body_block_14;

mod attack_secondary_body_block_15_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_15(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102052: cmpi.w #$1, $453794.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x453794_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010205a: bne.b $102072
        e.step()?;
        pc = if r.not_equal()? {
            0x102072_u32
        } else {
            0x10205c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_15_unit::attack_secondary_body_block_15;

mod attack_secondary_body_block_16_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_16(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010205c: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010205e: move.w $453798.l, -(a7)
        e.step()?;
        let value_address = 0x453798_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102064: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10206a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 0010206a: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 0010206c: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010206e: beq.b $102090
        e.step()?;
        pc = if r.equal()? {
            0x102090_u32
        } else {
            0x102070_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_16_unit::attack_secondary_body_block_16;

mod attack_secondary_body_block_17_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_17(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102070: bra.b $1020b4
        e.step()?;
        pc = 0x1020b4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_17_unit::attack_secondary_body_block_17;

mod attack_secondary_body_block_18_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_18(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102072: cmpi.w #$2, $453794.l
        e.step()?;
        let source = 0x2_u32;
        let destination_address = 0x453794_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010207a: bne.b $102090
        e.step()?;
        pc = if r.not_equal()? {
            0x102090_u32
        } else {
            0x10207c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_18_unit::attack_secondary_body_block_18;

mod attack_secondary_body_block_19_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_19(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010207c: move.w #$64, -(a7)
        e.step()?;
        let value = 0x64_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102080: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102086_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00102086: cmp.w $453798.l, d0
        e.step()?;
        let source_address = 0x453798_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010208c: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010208e: bgt.b $1020b4
        e.step()?;
        pc = if r.greater()? {
            0x1020b4_u32
        } else {
            0x102090_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_19_unit::attack_secondary_body_block_19;
mod attack_secondary_body_block_1a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_1a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102090: move.w #$1, -$d68(a6)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff298_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102096: move.w $45379a.l, -$d6a(a6)
        e.step()?;
        let value_address = 0x45379a_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffff296_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010209e: move.w $453796.l, -$d66(a6)
        e.step()?;
        let value_address = 0x453796_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffff29a_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001020a6: bra.b $1020b4
        e.step()?;
        pc = 0x1020b4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_1a_unit::attack_secondary_body_block_1a;

mod attack_secondary_body_block_1b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_1b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001020a8: bsr.l $136398
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1020ae_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x136398_u32)?;
        // 001020ae: clr.w $4537cc.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4537cc_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1020b4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_1b_unit::attack_secondary_body_block_1b;

mod attack_secondary_body_block_1c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_1c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001020b4: moveq #$5, d0
        e.step()?;
        r.put_data(0, 32, 0x5_u32);
        r.move_flags(0x5_u32, 32);
        // 001020b6: muls.w -$e0(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffff20_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001020ba: moveq #$5, d1
        e.step()?;
        r.put_data(1, 32, 0x5_u32);
        r.move_flags(0x5_u32, 32);
        // 001020bc: muls.w -$120(a6), d1
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffee0_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 001020c0: addi.w #$21, d1
        e.step()?;
        let source = 0x21_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 001020c4: add.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 001020c6: add.w d1, d6
        e.step()?;
        let source = r.data(1, 16)?;
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        // 001020c8: tst.b -$6c(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff94_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001020cc: beq.b $1020e0
        e.step()?;
        pc = if r.equal()? {
            0x1020e0_u32
        } else {
            0x1020ce_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_1c_unit::attack_secondary_body_block_1c;

mod attack_secondary_body_block_1d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_1d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001020ce: move.b -$6c(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001020d2: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001020d4: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001020d6: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1020dc_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 001020dc: sub.w d0, d6
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(6, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        // 001020de: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1020e0_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_1d_unit::attack_secondary_body_block_1d;

mod attack_secondary_body_block_1e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_1e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001020e0: tst.b -$68(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff98_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001020e4: beq.b $1020ea
        e.step()?;
        pc = if r.equal()? {
            0x1020ea_u32
        } else {
            0x1020e6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_1e_unit::attack_secondary_body_block_1e;

mod attack_secondary_body_block_1f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_1f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001020e6: subi.w #$f, d6
        e.step()?;
        let source = 0xf_u32;
        let destination = r.data(6, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        pc = 0x1020ea_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_1f_unit::attack_secondary_body_block_1f;

mod attack_secondary_body_block_20_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_20(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001020ea: tst.b -$53(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffad_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001020ee: beq.b $1020f4
        e.step()?;
        pc = if r.equal()? {
            0x1020f4_u32
        } else {
            0x1020f0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_20_unit::attack_secondary_body_block_20;

mod attack_secondary_body_block_21_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_21(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001020f0: subi.w #$f, d6
        e.step()?;
        let source = 0xf_u32;
        let destination = r.data(6, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        pc = 0x1020f4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_21_unit::attack_secondary_body_block_21;

mod attack_secondary_body_block_22_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_22(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001020f4: tst.b -$51(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffaf_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001020f8: beq.b $1020fe
        e.step()?;
        pc = if r.equal()? {
            0x1020fe_u32
        } else {
            0x1020fa_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_22_unit::attack_secondary_body_block_22;

mod attack_secondary_body_block_23_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_23(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001020fa: subi.w #$a, d6
        e.step()?;
        let source = 0xa_u32;
        let destination = r.data(6, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        pc = 0x1020fe_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_23_unit::attack_secondary_body_block_23;

mod attack_secondary_body_block_24_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_24(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001020fe: tst.b -$4a(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffb6_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00102102: beq.b $102116
        e.step()?;
        pc = if r.equal()? {
            0x102116_u32
        } else {
            0x102104_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_24_unit::attack_secondary_body_block_24;

mod attack_secondary_body_block_25_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_25(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102104: move.b -$4a(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffb6_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00102108: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010210a: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010210c: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102112_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102112: sub.w d0, d6
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(6, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        // 00102114: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x102116_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_25_unit::attack_secondary_body_block_25;

mod attack_secondary_body_block_26_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_26(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102116: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010211a: bge.w $102238
        e.step()?;
        pc = if r.greater_equal()? {
            0x102238_u32
        } else {
            0x10211e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_26_unit::attack_secondary_body_block_26;

mod attack_secondary_body_block_27_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_27(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010211e: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00102120: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102122: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00102128: lea.l $44709a.l, a1
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(1, value);
        // 0010212e: move.w (a1, d3.w * 4), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102132: cmp.w (a0, d0.w * 4), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102136: ble.b $10216e
        e.step()?;
        pc = if r.less_equal()? {
            0x10216e_u32
        } else {
            0x102138_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_27_unit::attack_secondary_body_block_27;

mod attack_secondary_body_block_28_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_28(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102138: tst.b -$19(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffe7_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0010213c: bne.b $10216e
        e.step()?;
        pc = if r.not_equal()? {
            0x10216e_u32
        } else {
            0x10213e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_28_unit::attack_secondary_body_block_28;

mod attack_secondary_body_block_29_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_29(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010213e: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00102140: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102142: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102146: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010214c: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010214e: move.b #$1, $107(a0)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x107_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00102154: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102156: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102158: bsr.l $105572
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10215e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x105572_u32)?;
        // 0010215e: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00102160: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102162: bsr.l $120c14
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102168_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x120c14_u32)?;
        // 00102168: lea.l $a(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xa_u32);
        r.put_address(7, value);
        // 0010216c: bra.b $1021bc
        e.step()?;
        pc = 0x1021bc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_29_unit::attack_secondary_body_block_29;
mod attack_secondary_body_block_2a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_2a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010216e: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00102170: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102172: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00102178: lea.l $44709a.l, a1
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(1, value);
        // 0010217e: move.w (a1, d3.w * 4), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102182: cmp.w (a0, d0.w * 4), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102186: bge.b $1021bc
        e.step()?;
        pc = if r.greater_equal()? {
            0x1021bc_u32
        } else {
            0x102188_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_2a_unit::attack_secondary_body_block_2a;

mod attack_secondary_body_block_2b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_2b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102188: cmpi.b #$1, -$19(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe7_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010218e: bne.b $1021bc
        e.step()?;
        pc = if r.not_equal()? {
            0x1021bc_u32
        } else {
            0x102190_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_2b_unit::attack_secondary_body_block_2b;

mod attack_secondary_body_block_2c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_2c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102190: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00102192: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102194: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102198: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010219e: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001021a0: clr.b $107(a0)
        e.step()?;
        let value = 0;
        let destination_address = r.address(0)?.wrapping_add(0x107_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001021a4: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001021a6: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001021a8: bsr.l $105572
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1021ae_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x105572_u32)?;
        // 001021ae: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001021b0: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001021b2: bsr.l $120c14
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1021b8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x120c14_u32)?;
        // 001021b8: lea.l $a(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xa_u32);
        r.put_address(7, value);
        pc = 0x1021bc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_2c_unit::attack_secondary_body_block_2c;

mod attack_secondary_body_block_2d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_2d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001021bc: tst.w -$d00(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff300_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001021c0: beq.b $1021d0
        e.step()?;
        pc = if r.equal()? {
            0x1021d0_u32
        } else {
            0x1021c2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_2d_unit::attack_secondary_body_block_2d;

mod attack_secondary_body_block_2e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_2e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001021c2: move.w -$d00(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff300_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001021c6: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1021cc_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 001021cc: add.w d0, d6
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        // 001021ce: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1021d0_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_2e_unit::attack_secondary_body_block_2e;

mod attack_secondary_body_block_2f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_2f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001021d0: tst.w -$cf8(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff308_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001021d4: beq.b $1021da
        e.step()?;
        pc = if r.equal()? {
            0x1021da_u32
        } else {
            0x1021d6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_2f_unit::attack_secondary_body_block_2f;

mod attack_secondary_body_block_30_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_30(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001021d6: addi.w #$f, d6
        e.step()?;
        let source = 0xf_u32;
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        pc = 0x1021da_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_30_unit::attack_secondary_body_block_30;

mod attack_secondary_body_block_31_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_31(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001021da: tst.w -$cce(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff332_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001021de: beq.b $1021e4
        e.step()?;
        pc = if r.equal()? {
            0x1021e4_u32
        } else {
            0x1021e0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_31_unit::attack_secondary_body_block_31;

mod attack_secondary_body_block_32_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_32(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001021e0: addi.w #$f, d6
        e.step()?;
        let source = 0xf_u32;
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        pc = 0x1021e4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_32_unit::attack_secondary_body_block_32;

mod attack_secondary_body_block_33_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_33(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001021e4: tst.w -$cca(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff336_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001021e8: beq.b $1021ee
        e.step()?;
        pc = if r.equal()? {
            0x1021ee_u32
        } else {
            0x1021ea_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_33_unit::attack_secondary_body_block_33;

mod attack_secondary_body_block_34_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_34(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001021ea: addi.w #$a, d6
        e.step()?;
        let source = 0xa_u32;
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        pc = 0x1021ee_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_34_unit::attack_secondary_body_block_34;

mod attack_secondary_body_block_35_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_35(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001021ee: tst.w -$cba(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff346_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001021f2: beq.b $102202
        e.step()?;
        pc = if r.equal()? {
            0x102202_u32
        } else {
            0x1021f4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_35_unit::attack_secondary_body_block_35;

mod attack_secondary_body_block_36_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_36(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001021f4: move.w -$cba(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff346_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001021f8: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1021fe_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 001021fe: add.w d0, d6
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        // 00102200: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x102202_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_36_unit::attack_secondary_body_block_36;

mod attack_secondary_body_block_37_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_37(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102202: tst.w -$cb8(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff348_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00102206: beq.b $102216
        e.step()?;
        pc = if r.equal()? {
            0x102216_u32
        } else {
            0x102208_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_37_unit::attack_secondary_body_block_37;

mod attack_secondary_body_block_38_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_38(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102208: move.w -$cb8(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff348_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010220c: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102212_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102212: sub.w d0, d6
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(6, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        // 00102214: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x102216_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_38_unit::attack_secondary_body_block_38;

mod attack_secondary_body_block_39_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_39(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102216: sub.w -$d58(a6), d6
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff2a8_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(6, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        // 0010221a: tst.b -$74(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff8c_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0010221e: bne.b $102228
        e.step()?;
        pc = if r.not_equal()? {
            0x102228_u32
        } else {
            0x102220_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_39_unit::attack_secondary_body_block_39;
mod attack_secondary_body_block_3a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_3a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102220: tst.b -$73(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff8d_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00102224: beq.w $10240c
        e.step()?;
        pc = if r.equal()? {
            0x10240c_u32
        } else {
            0x102228_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_3a_unit::attack_secondary_body_block_3a;

mod attack_secondary_body_block_3b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_3b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102228: tst.w -$cd8(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff328_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010222c: beq.w $10240c
        e.step()?;
        pc = if r.equal()? {
            0x10240c_u32
        } else {
            0x102230_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_3b_unit::attack_secondary_body_block_3b;

mod attack_secondary_body_block_3c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_3c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102230: subi.w #$a, d6
        e.step()?;
        let source = 0xa_u32;
        let destination = r.data(6, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        // 00102234: bra.w $10240c
        e.step()?;
        pc = 0x10240c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_3c_unit::attack_secondary_body_block_3c;

mod attack_secondary_body_block_3d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_3d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102238: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010223a: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010223c: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00102242: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00102244: add.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00102246: lea.l $446f0a.l, a1
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(1, value);
        // 0010224c: move.w (a1, d1.w * 4), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102250: cmp.w (a0, d0.w * 4), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102254: ble.b $10228c
        e.step()?;
        pc = if r.less_equal()? {
            0x10228c_u32
        } else {
            0x102256_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_3d_unit::attack_secondary_body_block_3d;

mod attack_secondary_body_block_3e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_3e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102256: tst.b -$19(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffe7_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0010225a: bne.b $10228c
        e.step()?;
        pc = if r.not_equal()? {
            0x10228c_u32
        } else {
            0x10225c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_3e_unit::attack_secondary_body_block_3e;

mod attack_secondary_body_block_3f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_3f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010225c: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010225e: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102260: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102264: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010226a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010226c: move.b #$1, $107(a0)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x107_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00102272: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102274: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102276: bsr.l $105572
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10227c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x105572_u32)?;
        // 0010227c: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010227e: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102280: bsr.l $120c14
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102286_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x120c14_u32)?;
        // 00102286: lea.l $a(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xa_u32);
        r.put_address(7, value);
        // 0010228a: bra.b $1022de
        e.step()?;
        pc = 0x1022de_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_3f_unit::attack_secondary_body_block_3f;

mod attack_secondary_body_block_40_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_40(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010228c: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010228e: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102290: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00102296: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00102298: add.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0010229a: lea.l $446f0a.l, a1
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(1, value);
        // 001022a0: move.w (a1, d1.w * 4), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001022a4: cmp.w (a0, d0.w * 4), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001022a8: bge.b $1022de
        e.step()?;
        pc = if r.greater_equal()? {
            0x1022de_u32
        } else {
            0x1022aa_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_40_unit::attack_secondary_body_block_40;

mod attack_secondary_body_block_41_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_41(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001022aa: cmpi.b #$1, -$19(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe7_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 001022b0: bne.b $1022de
        e.step()?;
        pc = if r.not_equal()? {
            0x1022de_u32
        } else {
            0x1022b2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_41_unit::attack_secondary_body_block_41;

mod attack_secondary_body_block_42_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_42(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001022b2: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001022b4: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001022b6: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001022ba: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001022c0: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001022c2: clr.b $107(a0)
        e.step()?;
        let value = 0;
        let destination_address = r.address(0)?.wrapping_add(0x107_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001022c6: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001022c8: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001022ca: bsr.l $105572
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1022d0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x105572_u32)?;
        // 001022d0: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001022d2: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001022d4: bsr.l $120c14
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1022da_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x120c14_u32)?;
        // 001022da: lea.l $a(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xa_u32);
        r.put_address(7, value);
        pc = 0x1022de_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_42_unit::attack_secondary_body_block_42;

mod attack_secondary_body_block_43_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_43(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001022de: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001022e0: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001022e2: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001022e6: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001022ec: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001022ee: tst.b $b4(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xb4_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001022f2: beq.b $102316
        e.step()?;
        pc = if r.equal()? {
            0x102316_u32
        } else {
            0x1022f4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_43_unit::attack_secondary_body_block_43;

mod attack_secondary_body_block_44_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_44(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001022f4: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001022f6: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001022f8: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001022fc: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102302: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102304: move.b $b4(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xb4_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00102308: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010230a: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010230c: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102312_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102312: add.w d0, d6
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        // 00102314: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x102316_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_44_unit::attack_secondary_body_block_44;

mod attack_secondary_body_block_45_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_45(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102316: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00102318: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010231a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010231e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102324: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102326: tst.b $b8(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xb8_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0010232a: beq.b $102330
        e.step()?;
        pc = if r.equal()? {
            0x102330_u32
        } else {
            0x10232c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_45_unit::attack_secondary_body_block_45;

mod attack_secondary_body_block_46_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_46(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010232c: addi.w #$f, d6
        e.step()?;
        let source = 0xf_u32;
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        pc = 0x102330_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_46_unit::attack_secondary_body_block_46;

mod attack_secondary_body_block_47_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_47(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102330: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00102332: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102334: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102338: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010233e: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102340: tst.b $cd(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xcd_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00102344: beq.b $10234a
        e.step()?;
        pc = if r.equal()? {
            0x10234a_u32
        } else {
            0x102346_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_47_unit::attack_secondary_body_block_47;

mod attack_secondary_body_block_48_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_48(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102346: addi.w #$f, d6
        e.step()?;
        let source = 0xf_u32;
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        pc = 0x10234a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_48_unit::attack_secondary_body_block_48;

mod attack_secondary_body_block_49_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_49(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010234a: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010234c: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010234e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102352: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102358: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010235a: tst.b $cf(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xcf_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0010235e: beq.b $102364
        e.step()?;
        pc = if r.equal()? {
            0x102364_u32
        } else {
            0x102360_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_49_unit::attack_secondary_body_block_49;
mod attack_secondary_body_block_4a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_4a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102360: addi.w #$a, d6
        e.step()?;
        let source = 0xa_u32;
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        pc = 0x102364_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_4a_unit::attack_secondary_body_block_4a;

mod attack_secondary_body_block_4b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_4b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102364: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00102366: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102368: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010236c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102372: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102374: tst.b $d7(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xd7_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00102378: beq.b $10239c
        e.step()?;
        pc = if r.equal()? {
            0x10239c_u32
        } else {
            0x10237a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_4b_unit::attack_secondary_body_block_4b;

mod attack_secondary_body_block_4c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_4c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010237a: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010237c: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010237e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102382: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102388: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010238a: move.b $d7(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xd7_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010238e: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102390: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102392: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102398_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102398: add.w d0, d6
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(6, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        // 0010239a: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x10239c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_4c_unit::attack_secondary_body_block_4c;

mod attack_secondary_body_block_4d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_4d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010239c: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010239e: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001023a0: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001023a4: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001023aa: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001023ac: tst.b $d8(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xd8_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001023b0: beq.b $1023d4
        e.step()?;
        pc = if r.equal()? {
            0x1023d4_u32
        } else {
            0x1023b2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_4d_unit::attack_secondary_body_block_4d;

mod attack_secondary_body_block_4e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_4e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001023b2: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001023b4: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001023b6: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001023ba: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001023c0: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001023c2: move.b $d8(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xd8_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001023c6: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001023c8: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001023ca: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1023d0_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 001023d0: sub.w d0, d6
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(6, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        // 001023d2: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1023d4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_4e_unit::attack_secondary_body_block_4e;

mod attack_secondary_body_block_4f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_4f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001023d4: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001023d6: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001023d8: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001023dc: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001023e2: sub.w $8(a0, d0.l), d6
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x8_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(6, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        // 001023e6: tst.b -$74(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff8c_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001023ea: bne.b $1023f2
        e.step()?;
        pc = if r.not_equal()? {
            0x1023f2_u32
        } else {
            0x1023ec_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_4f_unit::attack_secondary_body_block_4f;

mod attack_secondary_body_block_50_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_50(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001023ec: tst.b -$73(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff8d_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001023f0: beq.b $10240c
        e.step()?;
        pc = if r.equal()? {
            0x10240c_u32
        } else {
            0x1023f2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_50_unit::attack_secondary_body_block_50;

mod attack_secondary_body_block_51_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_51(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001023f2: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001023f4: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001023f6: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001023fa: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102400: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102402: tst.b $c8(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xc8_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00102406: beq.b $10240c
        e.step()?;
        pc = if r.equal()? {
            0x10240c_u32
        } else {
            0x102408_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_51_unit::attack_secondary_body_block_51;

mod attack_secondary_body_block_52_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_52(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102408: subi.w #$a, d6
        e.step()?;
        let source = 0xa_u32;
        let destination = r.data(6, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(6, 16, value);
        pc = 0x10240c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_52_unit::attack_secondary_body_block_52;

mod attack_secondary_body_block_53_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_53(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010240c: moveq #$5, d0
        e.step()?;
        r.put_data(0, 32, 0x5_u32);
        r.move_flags(0x5_u32, 32);
        // 0010240e: muls.w $442576.l, d0
        e.step()?;
        let source_address = 0x442576_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102414: moveq #$5a, d1
        e.step()?;
        r.put_data(1, 32, 0x5a_u32);
        r.move_flags(0x5a_u32, 32);
        // 00102416: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00102418: move.w d1, -(a7)
        e.step()?;
        let value = r.data(1, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010241a: moveq #$5, d0
        e.step()?;
        r.put_data(0, 32, 0x5_u32);
        r.move_flags(0x5_u32, 32);
        // 0010241c: muls.w $442576.l, d0
        e.step()?;
        let source_address = 0x442576_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102422: moveq #$f, d1
        e.step()?;
        r.put_data(1, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 00102424: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00102426: move.w d1, -(a7)
        e.step()?;
        let value = r.data(1, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102428: move.w d6, -(a7)
        e.step()?;
        let value = r.data(6, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010242a: bsr.l $13f65e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102430_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13f65e_u32)?;
        // 00102430: move.w d0, d6
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(6, 16, value);
        r.move_flags(value, 16);
        // 00102432: move.w #$64, -(a7)
        e.step()?;
        let value = 0x64_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102436: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10243c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 0010243c: cmp.w d6, d0
        e.step()?;
        let source = r.data(6, 16)?;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010243e: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102440: bgt.b $102448
        e.step()?;
        pc = if r.greater()? {
            0x102448_u32
        } else {
            0x102442_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_53_unit::attack_secondary_body_block_53;

mod attack_secondary_body_block_54_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_54(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102442: move.b #$1, -$d63(a6)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff29d_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x102448_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_54_unit::attack_secondary_body_block_54;

mod attack_secondary_body_block_55_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_55(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102448: tst.b -$d63(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff29d_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0010244c: beq.w $1024d8
        e.step()?;
        pc = if r.equal()? {
            0x1024d8_u32
        } else {
            0x102450_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_55_unit::attack_secondary_body_block_55;

mod attack_secondary_body_block_56_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_56(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102450: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102454: ble.w $1024d8
        e.step()?;
        pc = if r.less_equal()? {
            0x1024d8_u32
        } else {
            0x102458_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_56_unit::attack_secondary_body_block_56;

mod attack_secondary_body_block_57_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_57(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102458: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010245a: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010245c: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102460: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102466: tst.w $c(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0xc_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010246a: beq.b $1024d8
        e.step()?;
        pc = if r.equal()? {
            0x1024d8_u32
        } else {
            0x10246c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_57_unit::attack_secondary_body_block_57;

mod attack_secondary_body_block_58_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_58(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010246c: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010246e: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102470: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102474: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010247a: cmpi.w #$ffff, $c(a0, d0.l)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0xc_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102480: beq.b $1024d8
        e.step()?;
        pc = if r.equal()? {
            0x1024d8_u32
        } else {
            0x102482_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_58_unit::attack_secondary_body_block_58;

mod attack_secondary_body_block_59_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_59(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102482: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00102484: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102486: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010248a: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102490: cmpi.w #$fffe, $c(a0, d0.l)
        e.step()?;
        let source = 0xfffe_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0xc_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102496: beq.b $1024d8
        e.step()?;
        pc = if r.equal()? {
            0x1024d8_u32
        } else {
            0x102498_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_59_unit::attack_secondary_body_block_59;
mod attack_secondary_body_block_5a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_5a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102498: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010249a: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010249c: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001024a0: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001024a6: move.w $4537ce.l, d1
        e.step()?;
        let value_address = 0x4537ce_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001024ac: addi.w #$fc00, d1
        e.step()?;
        let source = 0xfc00_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 001024b0: cmp.w $c(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0xc_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001024b4: beq.b $1024d8
        e.step()?;
        pc = if r.equal()? {
            0x1024d8_u32
        } else {
            0x1024b6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_5a_unit::attack_secondary_body_block_5a;

mod attack_secondary_body_block_5b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_5b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001024b6: clr.w $4434b8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001024bc: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001024be: move.b d0, -$d63(a6)
        e.step()?;
        let value = r.data(0, 8)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff29d_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001024c2: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001024c4: move.w #$ffd3, -(a7)
        e.step()?;
        let value = 0xffd3_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001024c8: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001024ca: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1024d0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 001024d0: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001024d2: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001024d4: bra.w $103eac
        e.step()?;
        pc = 0x103eac_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_5b_unit::attack_secondary_body_block_5b;

mod attack_secondary_body_block_5c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_5c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001024d8: tst.b -$d63(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff29d_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001024dc: beq.w $102568
        e.step()?;
        pc = if r.equal()? {
            0x102568_u32
        } else {
            0x1024e0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_5c_unit::attack_secondary_body_block_5c;

mod attack_secondary_body_block_5d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_5d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001024e0: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001024e4: ble.w $102568
        e.step()?;
        pc = if r.less_equal()? {
            0x102568_u32
        } else {
            0x1024e8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_5d_unit::attack_secondary_body_block_5d;

mod attack_secondary_body_block_5e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_5e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001024e8: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001024ea: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001024ec: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001024f0: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001024f6: cmpi.w #$ffff, $c(a0, d0.l)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0xc_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001024fc: bne.b $102568
        e.step()?;
        pc = if r.not_equal()? {
            0x102568_u32
        } else {
            0x1024fe_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_5e_unit::attack_secondary_body_block_5e;

mod attack_secondary_body_block_5f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_5f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001024fe: cmpi.w #$ffff, $4537cc.l
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = 0x4537cc_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102506: beq.b $102568
        e.step()?;
        pc = if r.equal()? {
            0x102568_u32
        } else {
            0x102508_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_5f_unit::attack_secondary_body_block_5f;

mod attack_secondary_body_block_60_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_60(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102508: tst.w -$d2(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff2e_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010250c: beq.b $102568
        e.step()?;
        pc = if r.equal()? {
            0x102568_u32
        } else {
            0x10250e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_60_unit::attack_secondary_body_block_60;

mod attack_secondary_body_block_61_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_61(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010250e: clr.w $4434b8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102514: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00102516: move.b d0, -$d63(a6)
        e.step()?;
        let value = r.data(0, 8)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff29d_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 0010251a: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010251c: move.w #$ffce, -(a7)
        e.step()?;
        let value = 0xffce_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102520: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102522: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102528_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00102528: tst.w -$d2(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff2e_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010252c: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010252e: beq.b $102568
        e.step()?;
        pc = if r.equal()? {
            0x102568_u32
        } else {
            0x102530_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_61_unit::attack_secondary_body_block_61;

mod attack_secondary_body_block_62_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_62(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102530: clr.w -$d2(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffff2e_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102534: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00102536: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102538: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010253c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102542: clr.w $4e(a0, d0.l)
        e.step()?;
        let value = 0;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x4e_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102546: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102548: bsr.l $11f1a8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10254e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x11f1a8_u32)?;
        // 0010254e: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102550: move.w #$ffcc, -(a7)
        e.step()?;
        let value = 0xffcc_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102554: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102556: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10255c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 0010255c: clr.b -$d63(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffff29d_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00102560: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00102562: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102564: bra.w $103eac
        e.step()?;
        pc = 0x103eac_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_62_unit::attack_secondary_body_block_62;

mod attack_secondary_body_block_63_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_63(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102568: tst.b -$d63(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff29d_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0010256c: beq.w $1025f8
        e.step()?;
        pc = if r.equal()? {
            0x1025f8_u32
        } else {
            0x102570_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_63_unit::attack_secondary_body_block_63;

mod attack_secondary_body_block_64_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_64(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102570: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102574: ble.w $1025f8
        e.step()?;
        pc = if r.less_equal()? {
            0x1025f8_u32
        } else {
            0x102578_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_64_unit::attack_secondary_body_block_64;

mod attack_secondary_body_block_65_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_65(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102578: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010257a: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010257c: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102580: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102586: cmpi.w #$fffe, $c(a0, d0.l)
        e.step()?;
        let source = 0xfffe_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0xc_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010258c: bne.b $1025f8
        e.step()?;
        pc = if r.not_equal()? {
            0x1025f8_u32
        } else {
            0x10258e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_65_unit::attack_secondary_body_block_65;

mod attack_secondary_body_block_66_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_66(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010258e: cmpi.w #$fffe, $4537cc.l
        e.step()?;
        let source = 0xfffe_u32;
        let destination_address = 0x4537cc_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102596: beq.b $1025f8
        e.step()?;
        pc = if r.equal()? {
            0x1025f8_u32
        } else {
            0x102598_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_66_unit::attack_secondary_body_block_66;

mod attack_secondary_body_block_67_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_67(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102598: tst.w -$d2(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff2e_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010259c: beq.b $1025f8
        e.step()?;
        pc = if r.equal()? {
            0x1025f8_u32
        } else {
            0x10259e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_67_unit::attack_secondary_body_block_67;

mod attack_secondary_body_block_68_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_68(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010259e: clr.w $4434b8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001025a4: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001025a6: move.b d0, -$d63(a6)
        e.step()?;
        let value = r.data(0, 8)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff29d_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001025aa: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001025ac: move.w #$ffcd, -(a7)
        e.step()?;
        let value = 0xffcd_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001025b0: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001025b2: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1025b8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 001025b8: tst.w -$d2(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff2e_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001025bc: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001025be: beq.b $1025f8
        e.step()?;
        pc = if r.equal()? {
            0x1025f8_u32
        } else {
            0x1025c0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_68_unit::attack_secondary_body_block_68;

mod attack_secondary_body_block_69_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_69(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001025c0: clr.w -$d2(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffff2e_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001025c4: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001025c6: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001025c8: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001025cc: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001025d2: clr.w $4e(a0, d0.l)
        e.step()?;
        let value = 0;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x4e_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001025d6: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001025d8: bsr.l $11f1a8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1025de_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x11f1a8_u32)?;
        // 001025de: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001025e0: move.w #$ffcc, -(a7)
        e.step()?;
        let value = 0xffcc_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001025e4: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001025e6: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1025ec_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 001025ec: clr.b -$d63(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffff29d_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001025f0: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001025f2: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001025f4: bra.w $103eac
        e.step()?;
        pc = 0x103eac_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_69_unit::attack_secondary_body_block_69;
mod attack_secondary_body_block_6a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_6a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001025f8: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001025fc: bge.b $102650
        e.step()?;
        pc = if r.greater_equal()? {
            0x102650_u32
        } else {
            0x1025fe_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_6a_unit::attack_secondary_body_block_6a;

mod attack_secondary_body_block_6b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_6b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001025fe: cmpi.w #$ffff, d3
        e.step()?;
        let source = 0xffff_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102602: ble.b $102650
        e.step()?;
        pc = if r.less_equal()? {
            0x102650_u32
        } else {
            0x102604_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_6b_unit::attack_secondary_body_block_6b;

mod attack_secondary_body_block_6c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_6c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102604: cmpi.w #$9, d4
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102608: ble.b $102650
        e.step()?;
        pc = if r.less_equal()? {
            0x102650_u32
        } else {
            0x10260a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_6c_unit::attack_secondary_body_block_6c;

mod attack_secondary_body_block_6d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_6d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010260a: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010260c: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102610: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102616: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102618: tst.w $9c(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x9c_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010261c: beq.w $1026fc
        e.step()?;
        pc = if r.equal()? {
            0x1026fc_u32
        } else {
            0x102620_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_6d_unit::attack_secondary_body_block_6d;

mod attack_secondary_body_block_6e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_6e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102620: move.w #$64, -(a7)
        e.step()?;
        let value = 0x64_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102624: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10262a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 0010262a: cmpi.w #$22, d0
        e.step()?;
        let source = 0x22_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010262e: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102630: bge.w $1026fc
        e.step()?;
        pc = if r.greater_equal()? {
            0x1026fc_u32
        } else {
            0x102634_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_6e_unit::attack_secondary_body_block_6e;

mod attack_secondary_body_block_6f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_6f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102634: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00102636: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102638: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010263c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102642: tst.w $c(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0xc_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00102646: bne.w $1026fc
        e.step()?;
        pc = if r.not_equal()? {
            0x1026fc_u32
        } else {
            0x10264a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_6f_unit::attack_secondary_body_block_6f;

mod attack_secondary_body_block_70_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_70(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010264a: move.w d4, d3
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(3, 16, value);
        r.move_flags(value, 16);
        // 0010264c: bra.w $1026fc
        e.step()?;
        pc = 0x1026fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_70_unit::attack_secondary_body_block_70;

mod attack_secondary_body_block_71_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_71(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102650: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102654: ble.b $10269e
        e.step()?;
        pc = if r.less_equal()? {
            0x10269e_u32
        } else {
            0x102656_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_71_unit::attack_secondary_body_block_71;

mod attack_secondary_body_block_72_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_72(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102656: cmpi.w #$9, d4
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010265a: ble.b $10269e
        e.step()?;
        pc = if r.less_equal()? {
            0x10269e_u32
        } else {
            0x10265c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_72_unit::attack_secondary_body_block_72;

mod attack_secondary_body_block_73_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_73(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010265c: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010265e: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102660: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102664: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010266a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010266c: tst.b $d1(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xd1_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00102670: beq.w $1026fc
        e.step()?;
        pc = if r.equal()? {
            0x1026fc_u32
        } else {
            0x102674_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_73_unit::attack_secondary_body_block_73;

mod attack_secondary_body_block_74_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_74(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102674: move.w #$64, -(a7)
        e.step()?;
        let value = 0x64_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102678: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10267e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 0010267e: cmpi.w #$22, d0
        e.step()?;
        let source = 0x22_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102682: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102684: bge.b $1026fc
        e.step()?;
        pc = if r.greater_equal()? {
            0x1026fc_u32
        } else {
            0x102686_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_74_unit::attack_secondary_body_block_74;

mod attack_secondary_body_block_75_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_75(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102686: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00102688: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010268a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010268e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102694: tst.w $c(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0xc_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00102698: bne.b $1026fc
        e.step()?;
        pc = if r.not_equal()? {
            0x1026fc_u32
        } else {
            0x10269a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_75_unit::attack_secondary_body_block_75;

mod attack_secondary_body_block_76_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_76(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010269a: move.w d4, d3
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(3, 16, value);
        r.move_flags(value, 16);
        // 0010269c: bra.b $1026fc
        e.step()?;
        pc = 0x1026fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_76_unit::attack_secondary_body_block_76;

mod attack_secondary_body_block_77_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_77(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010269e: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001026a2: bge.b $1026c4
        e.step()?;
        pc = if r.greater_equal()? {
            0x1026c4_u32
        } else {
            0x1026a4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_77_unit::attack_secondary_body_block_77;

mod attack_secondary_body_block_78_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_78(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001026a4: tst.w -$d02(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff2fe_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001026a8: beq.b $1026c4
        e.step()?;
        pc = if r.equal()? {
            0x1026c4_u32
        } else {
            0x1026aa_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_78_unit::attack_secondary_body_block_78;

mod attack_secondary_body_block_79_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_79(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001026aa: move.w #$a, -(a7)
        e.step()?;
        let value = 0xa_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001026ae: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1026b4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 001026b4: add.w -$d12(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff2ee_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001026b8: move.w d0, $4434b8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001026be: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001026c0: bra.w $1028a4
        e.step()?;
        pc = 0x1028a4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_79_unit::attack_secondary_body_block_79;
mod attack_secondary_body_block_7a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_7a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001026c4: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001026c8: ble.b $1026fc
        e.step()?;
        pc = if r.less_equal()? {
            0x1026fc_u32
        } else {
            0x1026ca_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_7a_unit::attack_secondary_body_block_7a;

mod attack_secondary_body_block_7b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_7b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001026ca: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001026cc: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001026ce: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001026d2: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001026d8: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001026da: tst.b $b3(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xb3_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001026de: beq.b $1026fc
        e.step()?;
        pc = if r.equal()? {
            0x1026fc_u32
        } else {
            0x1026e0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_7b_unit::attack_secondary_body_block_7b;

mod attack_secondary_body_block_7c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_7c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001026e0: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001026e2: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001026e4: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001026e8: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001026ee: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001026f0: move.w $110(a0), $4434b8.l
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x110_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001026f8: bra.w $1028a4
        e.step()?;
        pc = 0x1028a4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_7c_unit::attack_secondary_body_block_7c;

mod attack_secondary_body_block_7d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_7d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001026fc: tst.b -$d63(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff29d_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00102700: beq.w $103dfc
        e.step()?;
        pc = if r.equal()? {
            0x103dfc_u32
        } else {
            0x102704_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_7d_unit::attack_secondary_body_block_7d;

mod attack_secondary_body_block_7e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_7e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102704: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102708: ble.b $102720
        e.step()?;
        pc = if r.less_equal()? {
            0x102720_u32
        } else {
            0x10270a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_7e_unit::attack_secondary_body_block_7e;

mod attack_secondary_body_block_7f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_7f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010270a: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010270c: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010270e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102712: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102718: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010271a: move.b #$1, $105(a0)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x105_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x102720_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_7f_unit::attack_secondary_body_block_7f;

mod attack_secondary_body_block_80_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_80(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102720: move.w -$e0(a6), $44376e.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff20_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x44376e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102728: move.b -$4e(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffb2_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010272c: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010272e: add.w d0, $44376e.l
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = 0x44376e_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00102734: move.w $44376e.l, $443770.l
        e.step()?;
        let value_address = 0x44376e_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x443770_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010273e: tst.w -$f8(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff08_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00102742: bne.w $1027f8
        e.step()?;
        pc = if r.not_equal()? {
            0x1027f8_u32
        } else {
            0x102746_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_80_unit::attack_secondary_body_block_80;

mod attack_secondary_body_block_81_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_81(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102746: movea.w $c(a6), a0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xc_u32);
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 0010274a: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010274c: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 0010274e: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 00102752: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102754: tst.b $93(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x93_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00102758: beq.b $1027be
        e.step()?;
        pc = if r.equal()? {
            0x1027be_u32
        } else {
            0x10275a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_81_unit::attack_secondary_body_block_81;

mod attack_secondary_body_block_82_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_82(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010275a: movea.w $c(a6), a0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xc_u32);
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 0010275e: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102760: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 00102762: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 00102766: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102768: move.b $93(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x93_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010276c: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010276e: add.w d0, $44376e.l
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = 0x44376e_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00102774: movea.w $c(a6), a0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xc_u32);
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 00102778: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010277a: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 0010277c: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 00102780: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102782: move.b $94(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00102786: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102788: add.w d0, $443770.l
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = 0x443770_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010278e: movea.w $c(a6), a0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xc_u32);
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 00102792: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102794: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 00102796: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 0010279a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010279c: move.b $95(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x95_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001027a0: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001027a2: move.w d0, $443774.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x443774_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001027a8: tst.w $443774.l
        e.step()?;
        let value_address = 0x443774_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001027ae: bne.w $102888
        e.step()?;
        pc = if r.not_equal()? {
            0x102888_u32
        } else {
            0x1027b2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_82_unit::attack_secondary_body_block_82;

mod attack_secondary_body_block_83_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_83(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001027b2: move.w #$64, $443774.l
        e.step()?;
        let value = 0x64_u32;
        let destination_address = 0x443774_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001027ba: bra.w $102888
        e.step()?;
        pc = 0x102888_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_83_unit::attack_secondary_body_block_83;

mod attack_secondary_body_block_84_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_84(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001027be: move.b -$8d(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff73_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001027c2: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001027c4: add.w d0, $44376e.l
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = 0x44376e_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001027ca: move.b -$8c(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff74_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001027ce: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001027d0: add.w d0, $443770.l
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = 0x443770_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001027d6: move.b -$8b(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff75_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001027da: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001027dc: move.w d0, $443774.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x443774_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001027e2: tst.w $443774.l
        e.step()?;
        let value_address = 0x443774_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001027e8: bne.w $102888
        e.step()?;
        pc = if r.not_equal()? {
            0x102888_u32
        } else {
            0x1027ec_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_84_unit::attack_secondary_body_block_84;

mod attack_secondary_body_block_85_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_85(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001027ec: move.w #$64, $443774.l
        e.step()?;
        let value = 0x64_u32;
        let destination_address = 0x443774_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001027f4: bra.w $102888
        e.step()?;
        pc = 0x102888_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_85_unit::attack_secondary_body_block_85;

mod attack_secondary_body_block_86_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_86(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001027f8: addq.w #$1, $44376e.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x44376e_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001027fe: move.w $453790.l, d0
        e.step()?;
        let value_address = 0x453790_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102804: add.w d0, $443770.l
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = 0x443770_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 0010280a: tst.w $4537f4.l
        e.step()?;
        let value_address = 0x4537f4_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00102810: beq.b $10281c
        e.step()?;
        pc = if r.equal()? {
            0x10281c_u32
        } else {
            0x102812_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_86_unit::attack_secondary_body_block_86;

mod attack_secondary_body_block_87_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_87(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102812: move.b $4537f5.l, $443765.l
        e.step()?;
        let value_address = 0x4537f5_u32;
        let value = u32::from(e.b(value_address)?);
        let destination_address = 0x443765_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x10281c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_87_unit::attack_secondary_body_block_87;

mod attack_secondary_body_block_88_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_88(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010281c: tst.w $4537a8.l
        e.step()?;
        let value_address = 0x4537a8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00102822: beq.b $10282e
        e.step()?;
        pc = if r.equal()? {
            0x10282e_u32
        } else {
            0x102824_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_88_unit::attack_secondary_body_block_88;

mod attack_secondary_body_block_89_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_89(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102824: move.b $4537a9.l, $443766.l
        e.step()?;
        let value_address = 0x4537a9_u32;
        let value = u32::from(e.b(value_address)?);
        let destination_address = 0x443766_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x10282e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_89_unit::attack_secondary_body_block_89;
mod attack_secondary_body_block_8a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_8a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010282e: tst.w $4537aa.l
        e.step()?;
        let value_address = 0x4537aa_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00102834: beq.b $102840
        e.step()?;
        pc = if r.equal()? {
            0x102840_u32
        } else {
            0x102836_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_8a_unit::attack_secondary_body_block_8a;

mod attack_secondary_body_block_8b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_8b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102836: move.b $4537ab.l, $443767.l
        e.step()?;
        let value_address = 0x4537ab_u32;
        let value = u32::from(e.b(value_address)?);
        let destination_address = 0x443767_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x102840_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_8b_unit::attack_secondary_body_block_8b;

mod attack_secondary_body_block_8c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_8c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102840: tst.w $4537ac.l
        e.step()?;
        let value_address = 0x4537ac_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00102846: beq.b $102852
        e.step()?;
        pc = if r.equal()? {
            0x102852_u32
        } else {
            0x102848_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_8c_unit::attack_secondary_body_block_8c;

mod attack_secondary_body_block_8d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_8d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102848: move.b $4537ad.l, $443768.l
        e.step()?;
        let value_address = 0x4537ad_u32;
        let value = u32::from(e.b(value_address)?);
        let destination_address = 0x443768_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x102852_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_8d_unit::attack_secondary_body_block_8d;

mod attack_secondary_body_block_8e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_8e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102852: tst.w $4537ae.l
        e.step()?;
        let value_address = 0x4537ae_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00102858: beq.b $102864
        e.step()?;
        pc = if r.equal()? {
            0x102864_u32
        } else {
            0x10285a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_8e_unit::attack_secondary_body_block_8e;

mod attack_secondary_body_block_8f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_8f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010285a: move.b $4537af.l, $443769.l
        e.step()?;
        let value_address = 0x4537af_u32;
        let value = u32::from(e.b(value_address)?);
        let destination_address = 0x443769_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x102864_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_8f_unit::attack_secondary_body_block_8f;

mod attack_secondary_body_block_90_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_90(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102864: tst.w $4537b0.l
        e.step()?;
        let value_address = 0x4537b0_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010286a: beq.b $102876
        e.step()?;
        pc = if r.equal()? {
            0x102876_u32
        } else {
            0x10286c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_90_unit::attack_secondary_body_block_90;

mod attack_secondary_body_block_91_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_91(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010286c: move.b $4537b1.l, $44376a.l
        e.step()?;
        let value_address = 0x4537b1_u32;
        let value = u32::from(e.b(value_address)?);
        let destination_address = 0x44376a_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x102876_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_91_unit::attack_secondary_body_block_91;

mod attack_secondary_body_block_92_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_92(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102876: tst.w $4537b2.l
        e.step()?;
        let value_address = 0x4537b2_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010287c: beq.b $102888
        e.step()?;
        pc = if r.equal()? {
            0x102888_u32
        } else {
            0x10287e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_92_unit::attack_secondary_body_block_92;

mod attack_secondary_body_block_93_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_93(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010287e: move.b $4537b3.l, $44376b.l
        e.step()?;
        let value_address = 0x4537b3_u32;
        let value = u32::from(e.b(value_address)?);
        let destination_address = 0x44376b_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x102888_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_93_unit::attack_secondary_body_block_93;

mod attack_secondary_body_block_94_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_94(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102888: move.l #$10000, -(a7)
        e.step()?;
        let value = 0x10000_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0010288e: pea.l $1.w
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00102892: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102894: bsr.l $15075a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10289a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x15075a_u32)?;
        // 0010289a: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 0010289c: lea.l $a(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xa_u32);
        r.put_address(7, value);
        // 001028a0: beq.w $103dfc
        e.step()?;
        pc = if r.equal()? {
            0x103dfc_u32
        } else {
            0x1028a4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_94_unit::attack_secondary_body_block_94;

mod attack_secondary_body_block_95_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_95(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001028a4: movea.w $c(a6), a0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xc_u32);
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 001028a8: move.l a0, d5
        e.step()?;
        let value = r.address(0)?;
        r.put_data(5, 32, value);
        r.move_flags(value, 32);
        // 001028aa: lsl.l #$2, d5
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(5, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(5, 32, value);
        // 001028ac: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 001028b0: adda.l d5, a0
        e.step()?;
        let source = r.data(5, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001028b2: move.b $96(a0), d5
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x96_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(5, 8, value);
        r.move_flags(value, 8);
        // 001028b6: ext.w d5
        e.step()?;
        let source = r.data(5, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(5, 16, value);
        r.move_flags(value, 16);
        // 001028b8: tst.w d5
        e.step()?;
        let value = r.data(5, 16)?;
        r.move_flags(value, 16);
        // 001028ba: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x1028be_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_95_unit::attack_secondary_body_block_95;

mod attack_secondary_body_block_96_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_96(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001028be: tst.w -$d2(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffff2e_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001028c2: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x1028c6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_96_unit::attack_secondary_body_block_96;

mod attack_secondary_body_block_97_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_97(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001028c6: move.w -$120(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffee0_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001028ca: move.w -$120(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffee0_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001028ce: lsr.w #$8, d0
        e.step()?;
        let count = 0x8_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(0, 16, value);
        // 001028d0: lsr.w #$7, d0
        e.step()?;
        let count = 0x7_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(0, 16, value);
        // 001028d2: add.w -$120(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffee0_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001028d6: asr.w #$1, d0
        e.step()?;
        let count = 0x1_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(0, 16, value);
        // 001028d8: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001028da: bsr.l $13849e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1028e0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13849e_u32)?;
        // 001028e0: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001028e6: cmpi.w #$1, $442bc8.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x442bc8_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001028ee: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001028f0: bge.b $1028fa
        e.step()?;
        pc = if r.greater_equal()? {
            0x1028fa_u32
        } else {
            0x1028f2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_97_unit::attack_secondary_body_block_97;

mod attack_secondary_body_block_98_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_98(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001028f2: move.w #$1, $442bc8.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1028fa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_98_unit::attack_secondary_body_block_98;

mod attack_secondary_body_block_99_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_99(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001028fa: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001028fe: bge.w $103344
        e.step()?;
        pc = if r.greater_equal()? {
            0x103344_u32
        } else {
            0x102902_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_99_unit::attack_secondary_body_block_99;
mod attack_secondary_body_block_9a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_9a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102902: cmpi.w #$ffff, d3
        e.step()?;
        let source = 0xffff_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102906: ble.w $103344
        e.step()?;
        pc = if r.less_equal()? {
            0x103344_u32
        } else {
            0x10290a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_9a_unit::attack_secondary_body_block_9a;

mod attack_secondary_body_block_9b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_9b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010290a: move.w d5, d0
        e.step()?;
        let value = r.data(5, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010290c: cmpi.w #$13, d0
        e.step()?;
        let source = 0x13_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102910: bhi.w $103c14
        e.step()?;
        pc = if r.higher()? {
            0x103c14_u32
        } else {
            0x102914_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_9b_unit::attack_secondary_body_block_9b;

mod attack_secondary_body_block_9c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_9c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102914: move.w $10291c(pc, d0.w), d0
        e.step()?;
        let selector = r.data(0, 16)? as usize;
        let offsets: [u16; 20] = [
            0x12f8, 0x0028, 0x00b6, 0x0166, 0x01dc, 0x0286, 0x0316, 0x03a6, 0x044c, 0x04a8, 0x053a,
            0x05a6, 0x0636, 0x06c6, 0x0756, 0x080a, 0x089a, 0x092a, 0x12f8, 0x09d6,
        ];
        let value = u32::from(*offsets.get(selector).ok_or_else(|| {
            RuleError::Invalid("source event selector outside recovered table".into())
        })?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102918: jmp $10291c(pc, d0.w)
        e.step()?;
        pc = 0x10291c_u32.wrapping_add(i32::from(r.data(0, 16)? as u16 as i16) as u32);

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_9c_unit::attack_secondary_body_block_9c;

mod attack_secondary_body_block_9d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_9d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102944: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102946: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102948: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10294e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 0010294e: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00102950: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102952: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x102956_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_9d_unit::attack_secondary_body_block_9d;

mod attack_secondary_body_block_9e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_9e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102956: cmpi.w #$ffff, -$d04(a6)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff2fc_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010295c: ble.w $103c14
        e.step()?;
        pc = if r.less_equal()? {
            0x103c14_u32
        } else {
            0x102960_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_9e_unit::attack_secondary_body_block_9e;

mod attack_secondary_body_block_9f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_9f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102960: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102962: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102966: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0010296c: cmpi.w #$3, $32(a0, d0.l)
        e.step()?;
        let source = 0x3_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x32_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102972: bne.b $10297a
        e.step()?;
        pc = if r.not_equal()? {
            0x10297a_u32
        } else {
            0x102974_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_9f_unit::attack_secondary_body_block_9f;

mod attack_secondary_body_block_a0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_a0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102974: clr.w $442bc8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x10297a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_a0_unit::attack_secondary_body_block_a0;

mod attack_secondary_body_block_a1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_a1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010297a: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102980: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102986_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102986: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102988: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0010298c: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102992: add.w $5e(a0, d1.l), d0
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x5e_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102996: cmpi.w #$1e, d0
        e.step()?;
        let source = 0x1e_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0010299a: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010299c: bge.b $1029be
        e.step()?;
        pc = if r.greater_equal()? {
            0x1029be_u32
        } else {
            0x10299e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_a1_unit::attack_secondary_body_block_a1;

mod attack_secondary_body_block_a2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_a2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010299e: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001029a4: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1029aa_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 001029aa: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001029ac: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 001029b0: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001029b6: lea.l $5e(a0, d1.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x5e_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 001029ba: add.w d0, (a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001029bc: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1029be_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_a2_unit::attack_secondary_body_block_a2;

mod attack_secondary_body_block_a3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_a3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001029be: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001029c0: move.w #$ffdf, -(a7)
        e.step()?;
        let value = 0xffdf_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001029c4: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001029c6: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1029cc_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 001029cc: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001029ce: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_a3_unit::attack_secondary_body_block_a3;

mod attack_secondary_body_block_a4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_a4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001029d2: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001029d4: move.w #$7, -(a7)
        e.step()?;
        let value = 0x7_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001029d8: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1029de_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 001029de: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 001029e0: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001029e2: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x1029e6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_a4_unit::attack_secondary_body_block_a4;

mod attack_secondary_body_block_a5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_a5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001029e6: cmpi.w #$ffff, -$d02(a6)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff2fe_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001029ec: ble.w $103c14
        e.step()?;
        pc = if r.less_equal()? {
            0x103c14_u32
        } else {
            0x1029f0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_a5_unit::attack_secondary_body_block_a5;

mod attack_secondary_body_block_a6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_a6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001029f0: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001029f2: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001029f6: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001029fc: cmpi.w #$3, $32(a0, d0.l)
        e.step()?;
        let source = 0x3_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x32_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102a02: bne.b $102a0a
        e.step()?;
        pc = if r.not_equal()? {
            0x102a0a_u32
        } else {
            0x102a04_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_a6_unit::attack_secondary_body_block_a6;

mod attack_secondary_body_block_a7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_a7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102a04: clr.w $442bc8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x102a0a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_a7_unit::attack_secondary_body_block_a7;

mod attack_secondary_body_block_a8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_a8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102a0a: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102a10: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102a16_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102a16: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102a18: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00102a1c: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102a22: add.w $60(a0, d1.l), d0
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x60_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102a26: cmpi.w #$1e, d0
        e.step()?;
        let source = 0x1e_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102a2a: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102a2c: bge.b $102a4e
        e.step()?;
        pc = if r.greater_equal()? {
            0x102a4e_u32
        } else {
            0x102a2e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_a8_unit::attack_secondary_body_block_a8;

mod attack_secondary_body_block_a9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_a9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102a2e: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102a34: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102a3a_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102a3a: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102a3c: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00102a40: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102a46: lea.l $60(a0, d1.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x60_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 00102a4a: add.w d0, (a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00102a4c: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x102a4e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_a9_unit::attack_secondary_body_block_a9;

mod attack_secondary_body_block_aa_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_aa(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102a4e: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102a50: move.w #$ffe0, -(a7)
        e.step()?;
        let value = 0xffe0_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102a54: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102a56: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102a5c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00102a5c: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102a5e: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102a62: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102a68: clr.w $3c(a0, d0.l)
        e.step()?;
        let value = 0;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x3c_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102a6c: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102a6e: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102a72: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102a78: clr.w $40(a0, d0.l)
        e.step()?;
        let value = 0;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x40_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102a7c: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102a7e: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_aa_unit::attack_secondary_body_block_aa;

mod attack_secondary_body_block_ab_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_ab(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102a82: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102a84: move.w #$5, -(a7)
        e.step()?;
        let value = 0x5_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102a88: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102a8e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00102a8e: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00102a90: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102a92: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x102a96_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_ab_unit::attack_secondary_body_block_ab;

mod attack_secondary_body_block_ac_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_ac(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102a96: cmpi.w #$ffff, -$cfa(a6)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff306_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102a9c: ble.w $103c14
        e.step()?;
        pc = if r.less_equal()? {
            0x103c14_u32
        } else {
            0x102aa0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_ac_unit::attack_secondary_body_block_ac;

mod attack_secondary_body_block_ad_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_ad(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102aa0: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102aa6: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102aac_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102aac: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102aae: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00102ab2: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102ab8: add.w $68(a0, d1.l), d0
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x68_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102abc: cmpi.w #$1e, d0
        e.step()?;
        let source = 0x1e_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102ac0: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102ac2: bge.b $102ae4
        e.step()?;
        pc = if r.greater_equal()? {
            0x102ae4_u32
        } else {
            0x102ac4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_ad_unit::attack_secondary_body_block_ad;

mod attack_secondary_body_block_ae_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_ae(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102ac4: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102aca: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102ad0_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102ad0: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102ad2: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00102ad6: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102adc: lea.l $68(a0, d1.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x68_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 00102ae0: add.w d0, (a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00102ae2: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x102ae4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_ae_unit::attack_secondary_body_block_ae;

mod attack_secondary_body_block_af_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_af(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102ae4: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102ae6: move.w #$ffe2, -(a7)
        e.step()?;
        let value = 0xffe2_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102aea: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102aec: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102af2_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00102af2: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102af4: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_af_unit::attack_secondary_body_block_af;

mod attack_secondary_body_block_b0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_b0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102af8: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102afa: move.w #$8, -(a7)
        e.step()?;
        let value = 0x8_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102afe: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102b04_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00102b04: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00102b06: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102b08: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x102b0c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_b0_unit::attack_secondary_body_block_b0;

mod attack_secondary_body_block_b1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_b1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102b0c: cmpi.w #$ffff, -$d00(a6)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff300_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102b12: ble.w $103c14
        e.step()?;
        pc = if r.less_equal()? {
            0x103c14_u32
        } else {
            0x102b16_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_b1_unit::attack_secondary_body_block_b1;

mod attack_secondary_body_block_b2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_b2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102b16: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102b1c: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102b22_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102b22: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102b24: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00102b28: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102b2e: add.w $62(a0, d1.l), d0
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x62_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102b32: cmpi.w #$1e, d0
        e.step()?;
        let source = 0x1e_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102b36: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102b38: bge.b $102b5a
        e.step()?;
        pc = if r.greater_equal()? {
            0x102b5a_u32
        } else {
            0x102b3a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_b2_unit::attack_secondary_body_block_b2;

mod attack_secondary_body_block_b3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_b3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102b3a: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102b40: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102b46_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102b46: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102b48: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00102b4c: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102b52: lea.l $62(a0, d1.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x62_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 00102b56: add.w d0, (a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00102b58: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x102b5a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_b3_unit::attack_secondary_body_block_b3;

mod attack_secondary_body_block_b4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_b4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102b5a: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102b5c: move.w #$ffe3, -(a7)
        e.step()?;
        let value = 0xffe3_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102b60: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102b62: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102b68_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00102b68: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102b6a: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102b6e: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102b74: lea.l $3c(a0, d0.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x3c_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 00102b78: move.w (a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x0_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102b7a: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102b7c: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00102b80: move.w d0, (a0)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102b82: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102b84: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102b88: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102b8e: lea.l $40(a0, d0.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x40_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 00102b92: move.w (a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x0_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102b94: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102b96: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00102b9a: move.w d0, (a0)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102b9c: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102b9e: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_b4_unit::attack_secondary_body_block_b4;

mod attack_secondary_body_block_b5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_b5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102ba2: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102ba4: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102ba8: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102bae_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00102bae: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00102bb0: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102bb2: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x102bb6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_b5_unit::attack_secondary_body_block_b5;

mod attack_secondary_body_block_b6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_b6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102bb6: cmpi.w #$ffff, -$cfe(a6)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff302_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102bbc: ble.w $103c14
        e.step()?;
        pc = if r.less_equal()? {
            0x103c14_u32
        } else {
            0x102bc0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_b6_unit::attack_secondary_body_block_b6;

mod attack_secondary_body_block_b7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_b7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102bc0: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102bc2: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102bc6: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102bcc: cmpi.w #$3, $32(a0, d0.l)
        e.step()?;
        let source = 0x3_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x32_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102bd2: bne.b $102bda
        e.step()?;
        pc = if r.not_equal()? {
            0x102bda_u32
        } else {
            0x102bd4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_b7_unit::attack_secondary_body_block_b7;

mod attack_secondary_body_block_b8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_b8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102bd4: clr.w $442bc8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x102bda_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_b8_unit::attack_secondary_body_block_b8;

mod attack_secondary_body_block_b9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_b9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102bda: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102be0: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102be6_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102be6: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102be8: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00102bec: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102bf2: add.w $64(a0, d1.l), d0
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x64_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102bf6: cmpi.w #$1e, d0
        e.step()?;
        let source = 0x1e_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102bfa: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102bfc: bge.b $102c1e
        e.step()?;
        pc = if r.greater_equal()? {
            0x102c1e_u32
        } else {
            0x102bfe_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_b9_unit::attack_secondary_body_block_b9;

mod attack_secondary_body_block_ba_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_ba(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102bfe: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102c04: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102c0a_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102c0a: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102c0c: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00102c10: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102c16: lea.l $64(a0, d1.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x64_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 00102c1a: add.w d0, (a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00102c1c: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x102c1e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_ba_unit::attack_secondary_body_block_ba;

mod attack_secondary_body_block_bb_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_bb(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102c1e: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102c20: move.w #$ffe4, -(a7)
        e.step()?;
        let value = 0xffe4_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102c24: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102c26: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102c2c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00102c2c: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102c2e: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_bb_unit::attack_secondary_body_block_bb;

mod attack_secondary_body_block_bc_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_bc(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102c32: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102c34: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102c36: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102c3c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00102c3c: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00102c3e: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102c40: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x102c44_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_bc_unit::attack_secondary_body_block_bc;

mod attack_secondary_body_block_bd_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_bd(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102c44: cmpi.w #$ffff, -$cca(a6)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff336_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102c4a: ble.w $103c14
        e.step()?;
        pc = if r.less_equal()? {
            0x103c14_u32
        } else {
            0x102c4e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_bd_unit::attack_secondary_body_block_bd;

mod attack_secondary_body_block_be_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_be(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102c4e: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102c50: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102c54: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102c5a: cmpi.w #$3, $32(a0, d0.l)
        e.step()?;
        let source = 0x3_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x32_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102c60: bne.b $102c68
        e.step()?;
        pc = if r.not_equal()? {
            0x102c68_u32
        } else {
            0x102c62_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_be_unit::attack_secondary_body_block_be;

mod attack_secondary_body_block_bf_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_bf(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102c62: clr.w $442bc8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x102c68_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_bf_unit::attack_secondary_body_block_bf;

mod attack_secondary_body_block_c0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_c0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102c68: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102c6e: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102c74_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102c74: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102c76: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00102c7a: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102c80: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102c82: add.w $98(a0), d0
        e.step()?;
        let source_address = r.address(0)?.wrapping_add(0x98_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102c86: cmpi.w #$1e, d0
        e.step()?;
        let source = 0x1e_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102c8a: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102c8c: bge.b $102cae
        e.step()?;
        pc = if r.greater_equal()? {
            0x102cae_u32
        } else {
            0x102c8e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_c0_unit::attack_secondary_body_block_c0;

mod attack_secondary_body_block_c1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_c1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102c8e: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102c94: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102c9a_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102c9a: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102c9c: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00102ca0: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102ca6: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102ca8: add.w d0, $98(a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x98_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00102cac: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x102cae_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_c1_unit::attack_secondary_body_block_c1;

mod attack_secondary_body_block_c2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_c2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102cae: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102cb0: move.w #$ffe5, -(a7)
        e.step()?;
        let value = 0xffe5_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102cb4: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102cb6: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102cbc_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00102cbc: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102cbe: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_c2_unit::attack_secondary_body_block_c2;

mod attack_secondary_body_block_c3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_c3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102cc2: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102cc4: move.w #$5, -(a7)
        e.step()?;
        let value = 0x5_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102cc8: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102cce_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00102cce: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00102cd0: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102cd2: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x102cd6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_c3_unit::attack_secondary_body_block_c3;

mod attack_secondary_body_block_c4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_c4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102cd6: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102cd8: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102cdc: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102ce2: tst.w $58(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x58_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00102ce6: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x102cea_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_c4_unit::attack_secondary_body_block_c4;

mod attack_secondary_body_block_c5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_c5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102cea: moveq #$3, d0
        e.step()?;
        r.put_data(0, 32, 0x3_u32);
        r.move_flags(0x3_u32, 32);
        // 00102cec: muls.w -$120(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffee0_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102cf0: move.w d0, $4434b6.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102cf6: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102cf8: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102cfc: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102d02: move.w $4434b6.l, d1
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102d08: cmp.w $58(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x58_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102d0c: ble.b $102d22
        e.step()?;
        pc = if r.less_equal()? {
            0x102d22_u32
        } else {
            0x102d0e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_c5_unit::attack_secondary_body_block_c5;

mod attack_secondary_body_block_c6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_c6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102d0e: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102d10: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102d14: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102d1a: move.w $58(a0, d0.l), $4434b6.l
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x58_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x102d22_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_c6_unit::attack_secondary_body_block_c6;

mod attack_secondary_body_block_c7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_c7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102d22: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102d24: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102d28: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102d2e: lea.l $58(a0, d0.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x58_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 00102d32: move.w $4434b6.l, d0
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102d38: sub.w d0, (a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00102d3a: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00102d3c: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102d3e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102d42: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00102d48: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102d4a: move.w $4434b6.l, d0
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102d50: add.w d0, $10e(a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x10e_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00102d54: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102d56: move.w #$ffef, -(a7)
        e.step()?;
        let value = 0xffef_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102d5a: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102d5c: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102d62_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00102d62: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102d64: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_c7_unit::attack_secondary_body_block_c7;

mod attack_secondary_body_block_c8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_c8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102d68: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102d6a: move.w #$8, -(a7)
        e.step()?;
        let value = 0x8_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102d6e: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102d74_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00102d74: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00102d76: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102d78: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x102d7c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_c8_unit::attack_secondary_body_block_c8;

mod attack_secondary_body_block_c9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_c9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102d7c: moveq #$14, d0
        e.step()?;
        r.put_data(0, 32, 0x14_u32);
        r.move_flags(0x14_u32, 32);
        // 00102d7e: muls.w -$e(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102d82: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102d84: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102d86: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00102d8a: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102d90: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102d92: add.l d0, $9ce(a0)
        e.step()?;
        let source = r.data(0, 32)?;
        let destination_address = r.address(0)?.wrapping_add(0x9ce_u32);
        let destination = e.l(destination_address)?;
        let value = r.add_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 00102d96: moveq #$14, d0
        e.step()?;
        r.put_data(0, 32, 0x14_u32);
        r.move_flags(0x14_u32, 32);
        // 00102d98: muls.w -$e(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102d9c: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102d9e: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102da0: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00102da4: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102daa: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102dac: sub.l d0, $9d2(a0)
        e.step()?;
        let source = r.data(0, 32)?;
        let destination_address = r.address(0)?.wrapping_add(0x9d2_u32);
        let destination = e.l(destination_address)?;
        let value = r.subtract_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 00102db0: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102db2: move.w #$ffee, -(a7)
        e.step()?;
        let value = 0xffee_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102db6: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102db8: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102dbe_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00102dbe: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102dc0: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_c9_unit::attack_secondary_body_block_c9;

mod attack_secondary_body_block_ca_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_ca(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102dc4: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102dc6: move.w #$7, -(a7)
        e.step()?;
        let value = 0x7_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102dca: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102dd0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00102dd0: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00102dd2: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102dd4: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x102dd8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_ca_unit::attack_secondary_body_block_ca;

mod attack_secondary_body_block_cb_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_cb(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102dd8: cmpi.w #$ffff, -$ccc(a6)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff334_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102dde: ble.w $103c14
        e.step()?;
        pc = if r.less_equal()? {
            0x103c14_u32
        } else {
            0x102de2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_cb_unit::attack_secondary_body_block_cb;

mod attack_secondary_body_block_cc_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_cc(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102de2: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102de4: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102de8: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102dee: cmpi.w #$3, $32(a0, d0.l)
        e.step()?;
        let source = 0x3_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x32_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102df4: bne.b $102dfc
        e.step()?;
        pc = if r.not_equal()? {
            0x102dfc_u32
        } else {
            0x102df6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_cc_unit::attack_secondary_body_block_cc;

mod attack_secondary_body_block_cd_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_cd(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102df6: clr.w $442bc8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x102dfc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_cd_unit::attack_secondary_body_block_cd;

mod attack_secondary_body_block_ce_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_ce(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102dfc: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102e02: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102e08_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102e08: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102e0a: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00102e0e: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102e14: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102e16: add.w $96(a0), d0
        e.step()?;
        let source_address = r.address(0)?.wrapping_add(0x96_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00102e1a: cmpi.w #$1e, d0
        e.step()?;
        let source = 0x1e_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00102e1e: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102e20: bge.b $102e42
        e.step()?;
        pc = if r.greater_equal()? {
            0x102e42_u32
        } else {
            0x102e22_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_ce_unit::attack_secondary_body_block_ce;

mod attack_secondary_body_block_cf_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_cf(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102e22: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102e28: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102e2e_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00102e2e: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00102e30: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00102e34: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102e3a: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102e3c: add.w d0, $96(a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x96_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00102e40: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x102e42_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_cf_unit::attack_secondary_body_block_cf;

mod attack_secondary_body_block_d0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_d0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102e42: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102e44: move.w #$ffde, -(a7)
        e.step()?;
        let value = 0xffde_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102e48: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102e4a: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102e50_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00102e50: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102e52: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_d0_unit::attack_secondary_body_block_d0;

mod attack_secondary_body_block_d1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_d1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102e56: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102e58: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102e5a: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102e60_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00102e60: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00102e62: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102e64: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x102e68_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_d1_unit::attack_secondary_body_block_d1;

mod attack_secondary_body_block_d2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_d2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102e68: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102e6a: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102e6e: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102e74: cmpi.w #$3, $32(a0, d0.l)
        e.step()?;
        let source = 0x3_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x32_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00102e7a: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x102e7e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_d2_unit::attack_secondary_body_block_d2;

mod attack_secondary_body_block_d3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_d3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102e7e: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102e80: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102e84: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102e8a: tst.w $10(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00102e8e: bne.b $102eac
        e.step()?;
        pc = if r.not_equal()? {
            0x102eac_u32
        } else {
            0x102e90_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_d3_unit::attack_secondary_body_block_d3;

mod attack_secondary_body_block_d4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_d4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102e90: addq.b #$1, $442739.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x442739_u32;
        let destination = u32::from(e.b(destination_address)?);
        let value = r.add_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        // 00102e96: addq.w #$1, $4435c4.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4435c4_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00102e9c: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102e9e: move.w #$ffe6, -(a7)
        e.step()?;
        let value = 0xffe6_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102ea2: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102ea4: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102eaa_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00102eaa: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x102eac_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_d4_unit::attack_secondary_body_block_d4;

mod attack_secondary_body_block_d5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_d5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102eac: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102eae: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102eb2: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102eb8: move.w -$112(a6), $10(a0, d0.l)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffeee_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102ebe: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_d5_unit::attack_secondary_body_block_d5;

mod attack_secondary_body_block_d6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_d6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102ec2: movea.w $4435d4.l, a0
        e.step()?;
        let value_address = 0x4435d4_u32;
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 00102ec8: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102eca: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 00102ecc: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 00102ed0: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102ed2: move.b $94(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00102ed6: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102ed8: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102eda: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102ee0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00102ee0: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102ee6: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102ee8: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102eec: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102ef2_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00102ef2: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00102ef4: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102ef6: beq.b $102f0a
        e.step()?;
        pc = if r.equal()? {
            0x102f0a_u32
        } else {
            0x102ef8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_d6_unit::attack_secondary_body_block_d6;

mod attack_secondary_body_block_d7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_d7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102ef8: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102efe: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102f00: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00102f04: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x102f0a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_d7_unit::attack_secondary_body_block_d7;

mod attack_secondary_body_block_d8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_d8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102f0a: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102f0c: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102f10: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102f16: tst.w $70(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x70_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00102f1a: beq.b $102f2e
        e.step()?;
        pc = if r.equal()? {
            0x102f2e_u32
        } else {
            0x102f1c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_d8_unit::attack_secondary_body_block_d8;

mod attack_secondary_body_block_d9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_d9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102f1c: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102f22: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102f24: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00102f28: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x102f2e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_d9_unit::attack_secondary_body_block_d9;

mod attack_secondary_body_block_da_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_da(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102f2e: add.w $442bc8.l, d7
        e.step()?;
        let source_address = 0x442bc8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(7, 16, value);
        // 00102f34: tst.w $442bc8.l
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00102f3a: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x102f3e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_da_unit::attack_secondary_body_block_da;

mod attack_secondary_body_block_db_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_db(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102f3e: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102f40: move.w #$ffea, -(a7)
        e.step()?;
        let value = 0xffea_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102f44: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102f46: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102f4c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00102f4c: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102f4e: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_db_unit::attack_secondary_body_block_db;

mod attack_secondary_body_block_dc_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_dc(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102f52: movea.w $4435d4.l, a0
        e.step()?;
        let value_address = 0x4435d4_u32;
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 00102f58: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102f5a: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 00102f5c: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 00102f60: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102f62: move.b $94(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00102f66: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102f68: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102f6a: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102f70_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00102f70: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102f76: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102f78: move.w #$2, -(a7)
        e.step()?;
        let value = 0x2_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102f7c: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102f82_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00102f82: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00102f84: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102f86: beq.b $102f9a
        e.step()?;
        pc = if r.equal()? {
            0x102f9a_u32
        } else {
            0x102f88_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_dc_unit::attack_secondary_body_block_dc;

mod attack_secondary_body_block_dd_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_dd(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102f88: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102f8e: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102f90: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00102f94: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x102f9a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_dd_unit::attack_secondary_body_block_dd;

mod attack_secondary_body_block_de_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_de(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102f9a: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102f9c: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102fa0: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00102fa6: tst.w $72(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x72_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00102faa: beq.b $102fbe
        e.step()?;
        pc = if r.equal()? {
            0x102fbe_u32
        } else {
            0x102fac_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_de_unit::attack_secondary_body_block_de;

mod attack_secondary_body_block_df_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_df(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102fac: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102fb2: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102fb4: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00102fb8: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x102fbe_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_df_unit::attack_secondary_body_block_df;

mod attack_secondary_body_block_e0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_e0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102fbe: add.w $442bc8.l, d7
        e.step()?;
        let source_address = 0x442bc8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(7, 16, value);
        // 00102fc4: tst.w $442bc8.l
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00102fca: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x102fce_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_e0_unit::attack_secondary_body_block_e0;

mod attack_secondary_body_block_e1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_e1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102fce: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102fd0: move.w #$ffed, -(a7)
        e.step()?;
        let value = 0xffed_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102fd4: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102fd6: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x102fdc_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00102fdc: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00102fde: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_e1_unit::attack_secondary_body_block_e1;

mod attack_secondary_body_block_e2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_e2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00102fe2: movea.w $4435d4.l, a0
        e.step()?;
        let value_address = 0x4435d4_u32;
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 00102fe8: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00102fea: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 00102fec: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 00102ff0: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00102ff2: move.b $94(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00102ff6: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00102ff8: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00102ffa: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103000_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00103000: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103006: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103008: move.w #$3, -(a7)
        e.step()?;
        let value = 0x3_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010300c: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103012_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00103012: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00103014: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103016: beq.b $10302a
        e.step()?;
        pc = if r.equal()? {
            0x10302a_u32
        } else {
            0x103018_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_e2_unit::attack_secondary_body_block_e2;

mod attack_secondary_body_block_e3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_e3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103018: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010301e: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103020: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00103024: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x10302a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_e3_unit::attack_secondary_body_block_e3;

mod attack_secondary_body_block_e4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_e4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010302a: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010302c: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103030: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103036: tst.w $74(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x74_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010303a: beq.b $10304e
        e.step()?;
        pc = if r.equal()? {
            0x10304e_u32
        } else {
            0x10303c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_e4_unit::attack_secondary_body_block_e4;

mod attack_secondary_body_block_e5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_e5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010303c: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103042: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103044: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00103048: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x10304e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_e5_unit::attack_secondary_body_block_e5;

mod attack_secondary_body_block_e6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_e6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010304e: add.w $442bc8.l, d7
        e.step()?;
        let source_address = 0x442bc8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(7, 16, value);
        // 00103054: tst.w $442bc8.l
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010305a: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x10305e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_e6_unit::attack_secondary_body_block_e6;

mod attack_secondary_body_block_e7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_e7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010305e: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103060: move.w #$ffec, -(a7)
        e.step()?;
        let value = 0xffec_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103064: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103066: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10306c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 0010306c: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010306e: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_e7_unit::attack_secondary_body_block_e7;

mod attack_secondary_body_block_e8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_e8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103072: movea.w $4435d4.l, a0
        e.step()?;
        let value_address = 0x4435d4_u32;
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 00103078: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010307a: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 0010307c: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 00103080: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103082: move.b $94(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00103086: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103088: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010308a: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103090_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00103090: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103096: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103098: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010309c: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001030a2: cmpi.w #$3, $32(a0, d0.l)
        e.step()?;
        let source = 0x3_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x32_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001030a8: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001030aa: bne.b $1030ba
        e.step()?;
        pc = if r.not_equal()? {
            0x1030ba_u32
        } else {
            0x1030ac_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_e8_unit::attack_secondary_body_block_e8;

mod attack_secondary_body_block_e9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_e9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001030ac: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001030b2: add.w d0, d0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001030b4: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1030ba_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_e9_unit::attack_secondary_body_block_e9;

mod attack_secondary_body_block_ea_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_ea(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001030ba: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001030bc: move.w #$4, -(a7)
        e.step()?;
        let value = 0x4_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001030c0: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1030c6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 001030c6: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 001030c8: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001030ca: beq.b $1030de
        e.step()?;
        pc = if r.equal()? {
            0x1030de_u32
        } else {
            0x1030cc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_ea_unit::attack_secondary_body_block_ea;

mod attack_secondary_body_block_eb_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_eb(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001030cc: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001030d2: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001030d4: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 001030d8: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1030de_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_eb_unit::attack_secondary_body_block_eb;

mod attack_secondary_body_block_ec_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_ec(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001030de: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001030e0: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001030e4: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001030ea: tst.w $76(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x76_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001030ee: beq.b $103102
        e.step()?;
        pc = if r.equal()? {
            0x103102_u32
        } else {
            0x1030f0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_ec_unit::attack_secondary_body_block_ec;

mod attack_secondary_body_block_ed_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_ed(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001030f0: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001030f6: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001030f8: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 001030fc: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x103102_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_ed_unit::attack_secondary_body_block_ed;

mod attack_secondary_body_block_ee_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_ee(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103102: add.w $442bc8.l, d7
        e.step()?;
        let source_address = 0x442bc8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(7, 16, value);
        // 00103108: tst.w $442bc8.l
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010310e: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x103112_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_ee_unit::attack_secondary_body_block_ee;

mod attack_secondary_body_block_ef_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_ef(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103112: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103114: move.w #$ffeb, -(a7)
        e.step()?;
        let value = 0xffeb_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103118: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010311a: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103120_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00103120: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103122: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_ef_unit::attack_secondary_body_block_ef;

mod attack_secondary_body_block_f0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_f0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103126: movea.w $4435d4.l, a0
        e.step()?;
        let value_address = 0x4435d4_u32;
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 0010312c: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010312e: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 00103130: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 00103134: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103136: move.b $94(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010313a: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010313c: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010313e: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103144_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00103144: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010314a: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010314c: move.w #$5, -(a7)
        e.step()?;
        let value = 0x5_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103150: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103156_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00103156: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00103158: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010315a: beq.b $10316e
        e.step()?;
        pc = if r.equal()? {
            0x10316e_u32
        } else {
            0x10315c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_f0_unit::attack_secondary_body_block_f0;

mod attack_secondary_body_block_f1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_f1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010315c: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103162: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103164: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00103168: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x10316e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_f1_unit::attack_secondary_body_block_f1;

mod attack_secondary_body_block_f2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_f2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010316e: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103170: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103174: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0010317a: tst.w $78(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x78_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010317e: beq.b $103192
        e.step()?;
        pc = if r.equal()? {
            0x103192_u32
        } else {
            0x103180_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_f2_unit::attack_secondary_body_block_f2;

mod attack_secondary_body_block_f3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_f3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103180: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103186: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103188: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 0010318c: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x103192_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_f3_unit::attack_secondary_body_block_f3;

mod attack_secondary_body_block_f4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_f4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103192: add.w $442bc8.l, d7
        e.step()?;
        let source_address = 0x442bc8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(7, 16, value);
        // 00103198: tst.w $442bc8.l
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010319e: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x1031a2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_f4_unit::attack_secondary_body_block_f4;

mod attack_secondary_body_block_f5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_f5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001031a2: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001031a4: move.w #$ffe9, -(a7)
        e.step()?;
        let value = 0xffe9_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001031a8: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001031aa: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1031b0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 001031b0: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001031b2: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_f5_unit::attack_secondary_body_block_f5;

mod attack_secondary_body_block_f6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_f6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001031b6: movea.w $4435d4.l, a0
        e.step()?;
        let value_address = 0x4435d4_u32;
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 001031bc: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001031be: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 001031c0: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 001031c4: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001031c6: move.b $94(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001031ca: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001031cc: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001031ce: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1031d4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 001031d4: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001031da: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001031dc: move.w #$6, -(a7)
        e.step()?;
        let value = 0x6_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001031e0: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1031e6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 001031e6: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 001031e8: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001031ea: beq.b $1031fe
        e.step()?;
        pc = if r.equal()? {
            0x1031fe_u32
        } else {
            0x1031ec_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_f6_unit::attack_secondary_body_block_f6;

mod attack_secondary_body_block_f7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_f7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001031ec: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001031f2: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001031f4: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 001031f8: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1031fe_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_f7_unit::attack_secondary_body_block_f7;

mod attack_secondary_body_block_f8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_f8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001031fe: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103200: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103204: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0010320a: tst.w $7a(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x7a_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010320e: beq.b $103222
        e.step()?;
        pc = if r.equal()? {
            0x103222_u32
        } else {
            0x103210_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_f8_unit::attack_secondary_body_block_f8;

mod attack_secondary_body_block_f9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_f9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103210: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103216: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103218: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 0010321c: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x103222_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_f9_unit::attack_secondary_body_block_f9;

mod attack_secondary_body_block_fa_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_fa(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103222: add.w $442bc8.l, d7
        e.step()?;
        let source_address = 0x442bc8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(7, 16, value);
        // 00103228: tst.w $442bc8.l
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010322e: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x103232_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_fa_unit::attack_secondary_body_block_fa;

mod attack_secondary_body_block_fb_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_fb(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103232: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103234: move.w #$fff1, -(a7)
        e.step()?;
        let value = 0xfff1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103238: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010323a: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103240_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00103240: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103242: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_fb_unit::attack_secondary_body_block_fb;

mod attack_secondary_body_block_fc_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_fc(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103246: movea.w $4435d4.l, a0
        e.step()?;
        let value_address = 0x4435d4_u32;
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 0010324c: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010324e: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 00103250: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 00103254: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103256: move.b $94(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010325a: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010325c: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010325e: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103264_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00103264: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010326a: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010326c: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103270: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103276: cmpi.w #$3, $32(a0, d0.l)
        e.step()?;
        let source = 0x3_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x32_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0010327c: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010327e: bne.b $103286
        e.step()?;
        pc = if r.not_equal()? {
            0x103286_u32
        } else {
            0x103280_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_fc_unit::attack_secondary_body_block_fc;

mod attack_secondary_body_block_fd_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_fd(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103280: clr.w $442bc8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x103286_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_fd_unit::attack_secondary_body_block_fd;

mod attack_secondary_body_block_fe_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_fe(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103286: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103288: move.w #$7, -(a7)
        e.step()?;
        let value = 0x7_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010328c: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103292_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00103292: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00103294: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103296: beq.b $1032aa
        e.step()?;
        pc = if r.equal()? {
            0x1032aa_u32
        } else {
            0x103298_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_fe_unit::attack_secondary_body_block_fe;

mod attack_secondary_body_block_ff_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_ff(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103298: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010329e: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001032a0: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 001032a4: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1032aa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_ff_unit::attack_secondary_body_block_ff;

mod attack_secondary_body_block_100_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_100(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001032aa: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001032ac: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001032b0: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001032b6: tst.w $7c(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x7c_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001032ba: beq.b $1032ce
        e.step()?;
        pc = if r.equal()? {
            0x1032ce_u32
        } else {
            0x1032bc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_100_unit::attack_secondary_body_block_100;

mod attack_secondary_body_block_101_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_101(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001032bc: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001032c2: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001032c4: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 001032c8: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1032ce_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_101_unit::attack_secondary_body_block_101;

mod attack_secondary_body_block_102_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_102(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001032ce: add.w $442bc8.l, d7
        e.step()?;
        let source_address = 0x442bc8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(7, 16, value);
        // 001032d4: tst.w $442bc8.l
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001032da: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x1032de_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_102_unit::attack_secondary_body_block_102;

mod attack_secondary_body_block_103_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_103(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001032de: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001032e0: move.w #$fff0, -(a7)
        e.step()?;
        let value = 0xfff0_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001032e4: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001032e6: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1032ec_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 001032ec: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001032ee: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_103_unit::attack_secondary_body_block_103;

mod attack_secondary_body_block_104_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_104(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001032f2: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001032f4: move.w #$8, -(a7)
        e.step()?;
        let value = 0x8_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001032f8: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1032fe_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 001032fe: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00103300: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103302: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x103306_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_104_unit::attack_secondary_body_block_104;

mod attack_secondary_body_block_105_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_105(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103306: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103308: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010330c: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103312: cmpi.w #$3, $32(a0, d0.l)
        e.step()?;
        let source = 0x3_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x32_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00103318: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x10331c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_105_unit::attack_secondary_body_block_105;

mod attack_secondary_body_block_106_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_106(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010331c: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010331e: move.w #$ffd1, -(a7)
        e.step()?;
        let value = 0xffd1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103322: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103324: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10332a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 0010332a: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010332c: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103330: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103336: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103338: move.w #$ffff, $94(a0)
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = r.address(0)?.wrapping_add(0x94_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010333e: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103340: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_106_unit::attack_secondary_body_block_106;

mod attack_secondary_body_block_107_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_107(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103344: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103346: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103348: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010334c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103352: cmpi.w #$64, $a(a0, d0.l)
        e.step()?;
        let source = 0x64_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0xa_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00103358: bgt.w $103dfc
        e.step()?;
        pc = if r.greater()? {
            0x103dfc_u32
        } else {
            0x10335c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_107_unit::attack_secondary_body_block_107;

mod attack_secondary_body_block_108_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_108(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010335c: move.w d5, d0
        e.step()?;
        let value = r.data(5, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010335e: cmpi.w #$13, d0
        e.step()?;
        let source = 0x13_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00103362: bhi.w $103c14
        e.step()?;
        pc = if r.higher()? {
            0x103c14_u32
        } else {
            0x103366_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_108_unit::attack_secondary_body_block_108;

mod attack_secondary_body_block_109_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_109(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103366: move.w $10336e(pc, d0.w), d0
        e.step()?;
        let selector = r.data(0, 16)? as usize;
        let offsets: [u16; 20] = [
            0x08a6, 0x0028, 0x0088, 0x0112, 0x0174, 0x0212, 0x0274, 0x02d4, 0x08a6, 0x0388, 0x03ea,
            0x0466, 0x04fa, 0x058e, 0x0622, 0x06b6, 0x074a, 0x07de, 0x08a6, 0x086e,
        ];
        let value = u32::from(*offsets.get(selector).ok_or_else(|| {
            RuleError::Invalid("source event selector outside recovered table".into())
        })?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010336a: jmp $10336e(pc, d0.w)
        e.step()?;
        pc = 0x10336e_u32.wrapping_add(i32::from(r.data(0, 16)? as u16 as i16) as u32);

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_109_unit::attack_secondary_body_block_109;
mod attack_secondary_body_block_10a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_10a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103396: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103398: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010339a: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1033a0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 001033a0: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 001033a2: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001033a4: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x1033a8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_10a_unit::attack_secondary_body_block_10a;

mod attack_secondary_body_block_10b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_10b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001033a8: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001033aa: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001033ac: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001033b0: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001033b6: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001033b8: cmpi.b #$ff, $b2(a0)
        e.step()?;
        let source = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xb2_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 001033be: ble.w $103c14
        e.step()?;
        pc = if r.less_equal()? {
            0x103c14_u32
        } else {
            0x1033c2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_10b_unit::attack_secondary_body_block_10b;

mod attack_secondary_body_block_10c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_10c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001033c2: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001033c8: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1033ce_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 001033ce: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001033d0: add.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 001033d2: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 001033d6: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001033dc: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001033de: add.b d0, $b2(a0)
        e.step()?;
        let source = r.data(0, 8)?;
        let destination_address = r.address(0)?.wrapping_add(0xb2_u32);
        let destination = u32::from(e.b(destination_address)?);
        let value = r.add_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        // 001033e2: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001033e4: move.w #$ffdf, -(a7)
        e.step()?;
        let value = 0xffdf_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001033e8: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001033ea: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1033f0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 001033f0: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001033f2: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_10c_unit::attack_secondary_body_block_10c;

mod attack_secondary_body_block_10d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_10d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001033f6: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001033f8: move.w #$7, -(a7)
        e.step()?;
        let value = 0x7_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001033fc: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103402_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00103402: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00103404: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103406: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x10340a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_10d_unit::attack_secondary_body_block_10d;

mod attack_secondary_body_block_10e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_10e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010340a: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010340c: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010340e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103412: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103418: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010341a: cmpi.b #$ff, $b3(a0)
        e.step()?;
        let source = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xb3_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00103420: ble.w $103c14
        e.step()?;
        pc = if r.less_equal()? {
            0x103c14_u32
        } else {
            0x103424_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_10e_unit::attack_secondary_body_block_10e;

mod attack_secondary_body_block_10f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_10f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103424: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010342a: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103430_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00103430: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103432: add.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00103434: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00103438: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010343e: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103440: add.b d0, $b3(a0)
        e.step()?;
        let source = r.data(0, 8)?;
        let destination_address = r.address(0)?.wrapping_add(0xb3_u32);
        let destination = u32::from(e.b(destination_address)?);
        let value = r.add_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        // 00103444: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103446: move.w #$ffe0, -(a7)
        e.step()?;
        let value = 0xffe0_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010344a: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010344c: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103452_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00103452: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103454: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103456: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010345a: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103460: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103462: clr.b $109(a0)
        e.step()?;
        let value = 0;
        let destination_address = r.address(0)?.wrapping_add(0x109_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00103466: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103468: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010346a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010346e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103474: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103476: clr.b $106(a0)
        e.step()?;
        let value = 0;
        let destination_address = r.address(0)?.wrapping_add(0x106_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 0010347a: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010347c: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_10f_unit::attack_secondary_body_block_10f;

mod attack_secondary_body_block_110_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_110(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103480: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103482: move.w #$5, -(a7)
        e.step()?;
        let value = 0x5_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103486: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10348c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 0010348c: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 0010348e: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103490: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x103494_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_110_unit::attack_secondary_body_block_110;

mod attack_secondary_body_block_111_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_111(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103494: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103496: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103498: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010349c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001034a2: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001034a4: cmpi.b #$ff, $b7(a0)
        e.step()?;
        let source = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xb7_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 001034aa: ble.w $103c14
        e.step()?;
        pc = if r.less_equal()? {
            0x103c14_u32
        } else {
            0x1034ae_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_111_unit::attack_secondary_body_block_111;

mod attack_secondary_body_block_112_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_112(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001034ae: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001034b4: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1034ba_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 001034ba: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001034bc: add.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 001034be: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 001034c2: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001034c8: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001034ca: add.b d0, $b7(a0)
        e.step()?;
        let source = r.data(0, 8)?;
        let destination_address = r.address(0)?.wrapping_add(0xb7_u32);
        let destination = u32::from(e.b(destination_address)?);
        let value = r.add_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        // 001034ce: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001034d0: move.w #$ffe2, -(a7)
        e.step()?;
        let value = 0xffe2_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001034d4: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001034d6: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1034dc_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 001034dc: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001034de: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_112_unit::attack_secondary_body_block_112;

mod attack_secondary_body_block_113_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_113(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001034e2: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001034e4: move.w #$8, -(a7)
        e.step()?;
        let value = 0x8_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001034e8: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1034ee_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 001034ee: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 001034f0: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001034f2: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x1034f6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_113_unit::attack_secondary_body_block_113;

mod attack_secondary_body_block_114_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_114(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001034f6: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001034f8: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001034fa: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001034fe: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103504: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103506: cmpi.b #$ff, $b4(a0)
        e.step()?;
        let source = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xb4_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010350c: ble.w $103c14
        e.step()?;
        pc = if r.less_equal()? {
            0x103c14_u32
        } else {
            0x103510_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_114_unit::attack_secondary_body_block_114;

mod attack_secondary_body_block_115_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_115(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103510: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103516: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10351c_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 0010351c: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010351e: add.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00103520: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00103524: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010352a: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010352c: add.b d0, $b4(a0)
        e.step()?;
        let source = r.data(0, 8)?;
        let destination_address = r.address(0)?.wrapping_add(0xb4_u32);
        let destination = u32::from(e.b(destination_address)?);
        let value = r.add_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        // 00103530: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103532: move.w #$ffe3, -(a7)
        e.step()?;
        let value = 0xffe3_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103536: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103538: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10353e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 0010353e: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103540: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103542: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103546: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010354c: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010354e: move.b $106(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x106_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00103552: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103554: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00103558: move.b d0, $106(a0)
        e.step()?;
        let value = r.data(0, 8)?;
        let destination_address = r.address(0)?.wrapping_add(0x106_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 0010355c: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010355e: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103560: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103564: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010356a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010356c: move.b $109(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x109_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00103570: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103572: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00103576: move.b d0, $109(a0)
        e.step()?;
        let value = r.data(0, 8)?;
        let destination_address = r.address(0)?.wrapping_add(0x109_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 0010357a: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010357c: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_115_unit::attack_secondary_body_block_115;

mod attack_secondary_body_block_116_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_116(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103580: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103582: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103586: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10358c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 0010358c: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 0010358e: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103590: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x103594_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_116_unit::attack_secondary_body_block_116;

mod attack_secondary_body_block_117_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_117(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103594: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103596: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103598: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010359c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001035a2: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001035a4: cmpi.b #$ff, $b5(a0)
        e.step()?;
        let source = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xb5_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 001035aa: ble.w $103c14
        e.step()?;
        pc = if r.less_equal()? {
            0x103c14_u32
        } else {
            0x1035ae_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_117_unit::attack_secondary_body_block_117;

mod attack_secondary_body_block_118_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_118(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001035ae: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001035b4: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1035ba_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 001035ba: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001035bc: add.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 001035be: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 001035c2: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001035c8: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001035ca: add.b d0, $b5(a0)
        e.step()?;
        let source = r.data(0, 8)?;
        let destination_address = r.address(0)?.wrapping_add(0xb5_u32);
        let destination = u32::from(e.b(destination_address)?);
        let value = r.add_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        // 001035ce: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001035d0: move.w #$ffe4, -(a7)
        e.step()?;
        let value = 0xffe4_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001035d4: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001035d6: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1035dc_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 001035dc: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001035de: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_118_unit::attack_secondary_body_block_118;

mod attack_secondary_body_block_119_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_119(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001035e2: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001035e4: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001035e6: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1035ec_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 001035ec: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 001035ee: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001035f0: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x1035f4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_119_unit::attack_secondary_body_block_119;
mod attack_secondary_body_block_11a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_11a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001035f4: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001035f6: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001035f8: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001035fc: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103602: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103604: cmpi.b #$ff, $cf(a0)
        e.step()?;
        let source = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xcf_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0010360a: ble.w $103c14
        e.step()?;
        pc = if r.less_equal()? {
            0x103c14_u32
        } else {
            0x10360e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_11a_unit::attack_secondary_body_block_11a;

mod attack_secondary_body_block_11b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_11b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010360e: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103614: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10361a_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 0010361a: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010361c: add.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0010361e: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00103622: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103628: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010362a: add.b d0, $cf(a0)
        e.step()?;
        let source = r.data(0, 8)?;
        let destination_address = r.address(0)?.wrapping_add(0xcf_u32);
        let destination = u32::from(e.b(destination_address)?);
        let value = r.add_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        // 0010362e: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103630: move.w #$ffe5, -(a7)
        e.step()?;
        let value = 0xffe5_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103634: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103636: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10363c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 0010363c: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010363e: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_11b_unit::attack_secondary_body_block_11b;

mod attack_secondary_body_block_11c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_11c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103642: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103644: move.w #$5, -(a7)
        e.step()?;
        let value = 0x5_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103648: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10364e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 0010364e: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00103650: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103652: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x103656_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_11c_unit::attack_secondary_body_block_11c;

mod attack_secondary_body_block_11d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_11d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103656: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103658: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010365a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010365e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103664: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103666: tst.w $10e(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x10e_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010366a: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x10366e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_11d_unit::attack_secondary_body_block_11d;

mod attack_secondary_body_block_11e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_11e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010366e: moveq #$3, d0
        e.step()?;
        r.put_data(0, 32, 0x3_u32);
        r.move_flags(0x3_u32, 32);
        // 00103670: muls.w -$120(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffffee0_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103674: move.w d0, $4434b6.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010367a: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010367c: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010367e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103682: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103688: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010368a: move.w $4434b6.l, d0
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103690: cmp.w $10e(a0), d0
        e.step()?;
        let source_address = r.address(0)?.wrapping_add(0x10e_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00103694: ble.b $1036ae
        e.step()?;
        pc = if r.less_equal()? {
            0x1036ae_u32
        } else {
            0x103696_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_11e_unit::attack_secondary_body_block_11e;

mod attack_secondary_body_block_11f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_11f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103696: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103698: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010369a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010369e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001036a4: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001036a6: move.w $10e(a0), $4434b6.l
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x10e_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4434b6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1036ae_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_11f_unit::attack_secondary_body_block_11f;

mod attack_secondary_body_block_120_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_120(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001036ae: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001036b0: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001036b2: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001036b6: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001036bc: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001036be: move.w $4434b6.l, d0
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001036c4: sub.w d0, $10e(a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x10e_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001036c8: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001036ca: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001036cc: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001036d0: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001036d6: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001036d8: move.w $4434b6.l, d0
        e.step()?;
        let value_address = 0x4434b6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001036de: add.w d0, $10e(a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x10e_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001036e2: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001036e4: move.w #$ffef, -(a7)
        e.step()?;
        let value = 0xffef_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001036e8: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001036ea: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1036f0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 001036f0: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001036f2: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_120_unit::attack_secondary_body_block_120;

mod attack_secondary_body_block_121_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_121(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001036f6: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001036f8: move.w #$7, -(a7)
        e.step()?;
        let value = 0x7_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001036fc: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103702_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00103702: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00103704: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103706: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x10370a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_121_unit::attack_secondary_body_block_121;

mod attack_secondary_body_block_122_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_122(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010370a: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010370c: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010370e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103712: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103718: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010371a: cmpi.b #$ff, $ce(a0)
        e.step()?;
        let source = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xce_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00103720: ble.w $103c14
        e.step()?;
        pc = if r.less_equal()? {
            0x103c14_u32
        } else {
            0x103724_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_122_unit::attack_secondary_body_block_122;

mod attack_secondary_body_block_123_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_123(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103724: move.w $442bc8.l, -(a7)
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010372a: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103730_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00103730: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103732: add.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00103734: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00103738: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010373e: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103740: add.b d0, $ce(a0)
        e.step()?;
        let source = r.data(0, 8)?;
        let destination_address = r.address(0)?.wrapping_add(0xce_u32);
        let destination = u32::from(e.b(destination_address)?);
        let value = r.add_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        // 00103744: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103746: move.w #$ffde, -(a7)
        e.step()?;
        let value = 0xffde_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010374a: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010374c: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103752_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00103752: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103754: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_123_unit::attack_secondary_body_block_123;

mod attack_secondary_body_block_124_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_124(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103758: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010375a: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010375c: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103762_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00103762: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00103764: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103766: bne.w $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x10376a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_124_unit::attack_secondary_body_block_124;

mod attack_secondary_body_block_125_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_125(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010376a: tst.w -$112(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffeee_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010376e: bne.b $103776
        e.step()?;
        pc = if r.not_equal()? {
            0x103776_u32
        } else {
            0x103770_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_125_unit::attack_secondary_body_block_125;

mod attack_secondary_body_block_126_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_126(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103770: subq.w #$1, $4435c4.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4435c4_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x103776_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_126_unit::attack_secondary_body_block_126;

mod attack_secondary_body_block_127_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_127(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103776: tst.w -$112(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffeee_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010377a: beq.b $103796
        e.step()?;
        pc = if r.equal()? {
            0x103796_u32
        } else {
            0x10377c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_127_unit::attack_secondary_body_block_127;

mod attack_secondary_body_block_128_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_128(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010377c: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010377e: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103780: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103784: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010378a: tst.w $e(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0010378e: bne.b $103796
        e.step()?;
        pc = if r.not_equal()? {
            0x103796_u32
        } else {
            0x103790_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_128_unit::attack_secondary_body_block_128;

mod attack_secondary_body_block_129_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_129(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103790: addq.w #$1, $4435c4.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4435c4_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x103796_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_129_unit::attack_secondary_body_block_129;
mod attack_secondary_body_block_12a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_12a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103796: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103798: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0010379a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010379e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001037a4: move.w -$112(a6), $e(a0, d0.l)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffeee_u32);
        let value = u32::from(e.w(value_address)?);
        let destination_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001037aa: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001037ac: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001037ae: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001037b2: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001037b8: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001037ba: move.w #$ffff, $10c(a0)
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = r.address(0)?.wrapping_add(0x10c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001037c0: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001037c2: move.w #$ffe6, -(a7)
        e.step()?;
        let value = 0xffe6_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001037c6: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001037c8: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1037ce_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 001037ce: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001037d0: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_12a_unit::attack_secondary_body_block_12a;

mod attack_secondary_body_block_12b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_12b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001037d4: movea.w $4435d4.l, a0
        e.step()?;
        let value_address = 0x4435d4_u32;
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 001037da: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001037dc: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 001037de: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 001037e2: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001037e4: move.b $94(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001037e8: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001037ea: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001037ec: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1037f2_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 001037f2: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001037f8: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001037fa: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001037fe: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103804_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00103804: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00103806: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103808: beq.b $10381c
        e.step()?;
        pc = if r.equal()? {
            0x10381c_u32
        } else {
            0x10380a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_12b_unit::attack_secondary_body_block_12b;

mod attack_secondary_body_block_12c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_12c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010380a: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103810: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103812: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00103816: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x10381c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_12c_unit::attack_secondary_body_block_12c;

mod attack_secondary_body_block_12d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_12d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010381c: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0010381e: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103820: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103824: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0010382a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010382c: tst.b $bb(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xbb_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00103830: beq.b $103844
        e.step()?;
        pc = if r.equal()? {
            0x103844_u32
        } else {
            0x103832_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_12d_unit::attack_secondary_body_block_12d;

mod attack_secondary_body_block_12e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_12e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103832: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103838: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010383a: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 0010383e: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x103844_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_12e_unit::attack_secondary_body_block_12e;

mod attack_secondary_body_block_12f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_12f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103844: add.w $442bc8.l, d7
        e.step()?;
        let source_address = 0x442bc8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(7, 16, value);
        // 0010384a: tst.w $442bc8.l
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00103850: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x103854_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_12f_unit::attack_secondary_body_block_12f;

mod attack_secondary_body_block_130_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_130(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103854: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103856: move.w #$ffea, -(a7)
        e.step()?;
        let value = 0xffea_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010385a: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010385c: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103862_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00103862: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103864: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_130_unit::attack_secondary_body_block_130;

mod attack_secondary_body_block_131_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_131(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103868: movea.w $4435d4.l, a0
        e.step()?;
        let value_address = 0x4435d4_u32;
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 0010386e: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103870: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 00103872: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 00103876: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103878: move.b $94(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0010387c: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0010387e: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103880: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103886_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00103886: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010388c: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010388e: move.w #$2, -(a7)
        e.step()?;
        let value = 0x2_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103892: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103898_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00103898: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 0010389a: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010389c: beq.b $1038b0
        e.step()?;
        pc = if r.equal()? {
            0x1038b0_u32
        } else {
            0x10389e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_131_unit::attack_secondary_body_block_131;

mod attack_secondary_body_block_132_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_132(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010389e: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001038a4: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001038a6: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 001038aa: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1038b0_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_132_unit::attack_secondary_body_block_132;

mod attack_secondary_body_block_133_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_133(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001038b0: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001038b2: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001038b4: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001038b8: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001038be: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001038c0: tst.b $bc(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xbc_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001038c4: beq.b $1038d8
        e.step()?;
        pc = if r.equal()? {
            0x1038d8_u32
        } else {
            0x1038c6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_133_unit::attack_secondary_body_block_133;

mod attack_secondary_body_block_134_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_134(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001038c6: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001038cc: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001038ce: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 001038d2: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1038d8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_134_unit::attack_secondary_body_block_134;

mod attack_secondary_body_block_135_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_135(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001038d8: add.w $442bc8.l, d7
        e.step()?;
        let source_address = 0x442bc8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(7, 16, value);
        // 001038de: tst.w $442bc8.l
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001038e4: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x1038e8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_135_unit::attack_secondary_body_block_135;

mod attack_secondary_body_block_136_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_136(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001038e8: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001038ea: move.w #$ffed, -(a7)
        e.step()?;
        let value = 0xffed_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001038ee: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001038f0: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1038f6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 001038f6: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001038f8: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_136_unit::attack_secondary_body_block_136;

mod attack_secondary_body_block_137_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_137(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001038fc: movea.w $4435d4.l, a0
        e.step()?;
        let value_address = 0x4435d4_u32;
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 00103902: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103904: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 00103906: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 0010390a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0010390c: move.b $94(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00103910: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103912: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103914: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10391a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 0010391a: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103920: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103922: move.w #$3, -(a7)
        e.step()?;
        let value = 0x3_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103926: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10392c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 0010392c: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 0010392e: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103930: beq.b $103944
        e.step()?;
        pc = if r.equal()? {
            0x103944_u32
        } else {
            0x103932_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_137_unit::attack_secondary_body_block_137;

mod attack_secondary_body_block_138_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_138(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103932: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103938: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010393a: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 0010393e: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x103944_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_138_unit::attack_secondary_body_block_138;

mod attack_secondary_body_block_139_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_139(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103944: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103946: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103948: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0010394c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103952: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103954: tst.b $bd(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xbd_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00103958: beq.b $10396c
        e.step()?;
        pc = if r.equal()? {
            0x10396c_u32
        } else {
            0x10395a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_139_unit::attack_secondary_body_block_139;
mod attack_secondary_body_block_13a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_13a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010395a: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103960: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103962: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00103966: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x10396c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_13a_unit::attack_secondary_body_block_13a;

mod attack_secondary_body_block_13b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_13b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010396c: add.w $442bc8.l, d7
        e.step()?;
        let source_address = 0x442bc8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(7, 16, value);
        // 00103972: tst.w $442bc8.l
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00103978: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x10397c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_13b_unit::attack_secondary_body_block_13b;

mod attack_secondary_body_block_13c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_13c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0010397c: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0010397e: move.w #$ffec, -(a7)
        e.step()?;
        let value = 0xffec_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103982: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103984: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x10398a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 0010398a: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0010398c: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_13c_unit::attack_secondary_body_block_13c;

mod attack_secondary_body_block_13d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_13d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103990: movea.w $4435d4.l, a0
        e.step()?;
        let value_address = 0x4435d4_u32;
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 00103996: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103998: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 0010399a: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 0010399e: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001039a0: move.b $94(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001039a4: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001039a6: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001039a8: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1039ae_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 001039ae: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001039b4: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001039b6: move.w #$4, -(a7)
        e.step()?;
        let value = 0x4_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001039ba: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1039c0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 001039c0: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 001039c2: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001039c4: beq.b $1039d8
        e.step()?;
        pc = if r.equal()? {
            0x1039d8_u32
        } else {
            0x1039c6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_13d_unit::attack_secondary_body_block_13d;

mod attack_secondary_body_block_13e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_13e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001039c6: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001039cc: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001039ce: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 001039d2: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1039d8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_13e_unit::attack_secondary_body_block_13e;

mod attack_secondary_body_block_13f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_13f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001039d8: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001039da: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001039dc: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001039e0: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001039e6: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001039e8: tst.b $be(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xbe_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001039ec: beq.b $103a00
        e.step()?;
        pc = if r.equal()? {
            0x103a00_u32
        } else {
            0x1039ee_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_13f_unit::attack_secondary_body_block_13f;

mod attack_secondary_body_block_140_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_140(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001039ee: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001039f4: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001039f6: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 001039fa: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x103a00_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_140_unit::attack_secondary_body_block_140;

mod attack_secondary_body_block_141_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_141(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103a00: add.w $442bc8.l, d7
        e.step()?;
        let source_address = 0x442bc8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(7, 16, value);
        // 00103a06: tst.w $442bc8.l
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00103a0c: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x103a10_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_141_unit::attack_secondary_body_block_141;

mod attack_secondary_body_block_142_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_142(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103a10: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103a12: move.w #$ffeb, -(a7)
        e.step()?;
        let value = 0xffeb_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103a16: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103a18: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103a1e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00103a1e: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103a20: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_142_unit::attack_secondary_body_block_142;

mod attack_secondary_body_block_143_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_143(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103a24: movea.w $4435d4.l, a0
        e.step()?;
        let value_address = 0x4435d4_u32;
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 00103a2a: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103a2c: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 00103a2e: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 00103a32: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103a34: move.b $94(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00103a38: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103a3a: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103a3c: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103a42_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00103a42: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103a48: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103a4a: move.w #$5, -(a7)
        e.step()?;
        let value = 0x5_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103a4e: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103a54_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00103a54: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00103a56: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103a58: beq.b $103a6c
        e.step()?;
        pc = if r.equal()? {
            0x103a6c_u32
        } else {
            0x103a5a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_143_unit::attack_secondary_body_block_143;

mod attack_secondary_body_block_144_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_144(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103a5a: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103a60: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103a62: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00103a66: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x103a6c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_144_unit::attack_secondary_body_block_144;

mod attack_secondary_body_block_145_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_145(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103a6c: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103a6e: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103a70: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103a74: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103a7a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103a7c: tst.b $bf(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xbf_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00103a80: beq.b $103a94
        e.step()?;
        pc = if r.equal()? {
            0x103a94_u32
        } else {
            0x103a82_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_145_unit::attack_secondary_body_block_145;

mod attack_secondary_body_block_146_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_146(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103a82: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103a88: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103a8a: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00103a8e: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x103a94_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_146_unit::attack_secondary_body_block_146;

mod attack_secondary_body_block_147_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_147(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103a94: add.w $442bc8.l, d7
        e.step()?;
        let source_address = 0x442bc8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(7, 16, value);
        // 00103a9a: tst.w $442bc8.l
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00103aa0: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x103aa4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_147_unit::attack_secondary_body_block_147;

mod attack_secondary_body_block_148_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_148(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103aa4: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103aa6: move.w #$ffe9, -(a7)
        e.step()?;
        let value = 0xffe9_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103aaa: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103aac: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103ab2_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00103ab2: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103ab4: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_148_unit::attack_secondary_body_block_148;

mod attack_secondary_body_block_149_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_149(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103ab8: movea.w $4435d4.l, a0
        e.step()?;
        let value_address = 0x4435d4_u32;
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 00103abe: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103ac0: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 00103ac2: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 00103ac6: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103ac8: move.b $94(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00103acc: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103ace: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103ad0: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103ad6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00103ad6: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103adc: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103ade: move.w #$6, -(a7)
        e.step()?;
        let value = 0x6_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103ae2: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103ae8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00103ae8: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00103aea: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103aec: beq.b $103b00
        e.step()?;
        pc = if r.equal()? {
            0x103b00_u32
        } else {
            0x103aee_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_149_unit::attack_secondary_body_block_149;
mod attack_secondary_body_block_14a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_14a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103aee: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103af4: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103af6: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00103afa: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x103b00_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_14a_unit::attack_secondary_body_block_14a;

mod attack_secondary_body_block_14b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_14b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103b00: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103b02: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103b04: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103b08: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103b0e: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103b10: tst.b $c0(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xc0_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00103b14: beq.b $103b28
        e.step()?;
        pc = if r.equal()? {
            0x103b28_u32
        } else {
            0x103b16_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_14b_unit::attack_secondary_body_block_14b;

mod attack_secondary_body_block_14c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_14c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103b16: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103b1c: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103b1e: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00103b22: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x103b28_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_14c_unit::attack_secondary_body_block_14c;

mod attack_secondary_body_block_14d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_14d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103b28: add.w $442bc8.l, d7
        e.step()?;
        let source_address = 0x442bc8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(7, 16, value);
        // 00103b2e: tst.w $442bc8.l
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00103b34: beq.w $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x103b38_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_14d_unit::attack_secondary_body_block_14d;

mod attack_secondary_body_block_14e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_14e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103b38: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103b3a: move.w #$fff1, -(a7)
        e.step()?;
        let value = 0xfff1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103b3e: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103b40: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103b46_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00103b46: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103b48: bra.w $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_14e_unit::attack_secondary_body_block_14e;

mod attack_secondary_body_block_14f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_14f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103b4c: movea.w $4435d4.l, a0
        e.step()?;
        let value_address = 0x4435d4_u32;
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 00103b52: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103b54: lsl.l #$2, d0
        e.step()?;
        let count = 0x2_u32;
        let source = r.data(0, 32)?;
        let value = r.shift_left(source, count & 63, 32);
        r.put_data(0, 32, value);
        // 00103b56: lea.l -$120(a6), a0
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xfffffee0_u32);
        r.put_address(0, value);
        // 00103b5a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103b5c: move.b $94(a0), d0
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x94_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00103b60: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103b62: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103b64: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103b6a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00103b6a: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103b70: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103b72: move.w #$7, -(a7)
        e.step()?;
        let value = 0x7_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103b76: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103b7c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00103b7c: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00103b7e: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103b80: beq.b $103b94
        e.step()?;
        pc = if r.equal()? {
            0x103b94_u32
        } else {
            0x103b82_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_14f_unit::attack_secondary_body_block_14f;

mod attack_secondary_body_block_150_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_150(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103b82: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103b88: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103b8a: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00103b8e: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x103b94_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_150_unit::attack_secondary_body_block_150;

mod attack_secondary_body_block_151_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_151(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103b94: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103b96: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103b98: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103b9c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103ba2: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103ba4: tst.b $c1(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xc1_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00103ba8: beq.b $103bbc
        e.step()?;
        pc = if r.equal()? {
            0x103bbc_u32
        } else {
            0x103baa_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_151_unit::attack_secondary_body_block_151;

mod attack_secondary_body_block_152_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_152(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103baa: move.w $442bc8.l, d0
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103bb0: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103bb2: divs.w #$2, d0
        e.step()?;
        let divisor = 0x2_u32;
        let dividend = r.data(0, 32)?;
        let value = r.divide_signed(dividend, divisor)?;
        r.put_data(0, 32, value);
        // 00103bb6: move.w d0, $442bc8.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x103bbc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_152_unit::attack_secondary_body_block_152;

mod attack_secondary_body_block_153_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_153(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103bbc: add.w $442bc8.l, d7
        e.step()?;
        let source_address = 0x442bc8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(7, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(7, 16, value);
        // 00103bc2: tst.w $442bc8.l
        e.step()?;
        let value_address = 0x442bc8_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00103bc8: beq.b $103c14
        e.step()?;
        pc = if r.equal()? {
            0x103c14_u32
        } else {
            0x103bca_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_153_unit::attack_secondary_body_block_153;

mod attack_secondary_body_block_154_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_154(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103bca: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103bcc: move.w #$fff0, -(a7)
        e.step()?;
        let value = 0xfff0_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103bd0: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103bd2: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103bd8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00103bd8: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103bda: bra.b $103c14
        e.step()?;
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_154_unit::attack_secondary_body_block_154;

mod attack_secondary_body_block_155_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_155(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103bdc: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103bde: move.w #$8, -(a7)
        e.step()?;
        let value = 0x8_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103be2: bsr.l $13a3ae
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103be8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13a3ae_u32)?;
        // 00103be8: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00103bea: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103bec: bne.b $103c14
        e.step()?;
        pc = if r.not_equal()? {
            0x103c14_u32
        } else {
            0x103bee_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_155_unit::attack_secondary_body_block_155;

mod attack_secondary_body_block_156_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_156(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103bee: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103bf0: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103bf2: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103bf6: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103bfc: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103bfe: move.b #$ff, $cd(a0)
        e.step()?;
        let value = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xcd_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00103c04: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103c06: move.w #$ffd1, -(a7)
        e.step()?;
        let value = 0xffd1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103c0a: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103c0c: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103c12_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00103c12: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x103c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_156_unit::attack_secondary_body_block_156;

mod attack_secondary_body_block_157_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_157(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103c14: tst.w $442566.l
        e.step()?;
        let value_address = 0x442566_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00103c1a: beq.b $103c50
        e.step()?;
        pc = if r.equal()? {
            0x103c50_u32
        } else {
            0x103c1c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_157_unit::attack_secondary_body_block_157;

mod attack_secondary_body_block_158_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_158(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103c1c: cmpi.w #$1, $4434b8.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4434b8_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00103c24: ble.b $103c50
        e.step()?;
        pc = if r.less_equal()? {
            0x103c50_u32
        } else {
            0x103c26_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_158_unit::attack_secondary_body_block_158;

mod attack_secondary_body_block_159_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_159(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103c26: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00103c2a: bge.b $103c50
        e.step()?;
        pc = if r.greater_equal()? {
            0x103c50_u32
        } else {
            0x103c2c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_159_unit::attack_secondary_body_block_159;
mod attack_secondary_body_block_15a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_15a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103c2c: subq.w #$5, $4434b8.l
        e.step()?;
        let source = 0x5_u32;
        let destination_address = 0x4434b8_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00103c32: cmpi.w #$1, $4434b8.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4434b8_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00103c3a: bge.b $103c44
        e.step()?;
        pc = if r.greater_equal()? {
            0x103c44_u32
        } else {
            0x103c3c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_15a_unit::attack_secondary_body_block_15a;

mod attack_secondary_body_block_15b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_15b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103c3c: move.w #$1, $4434b8.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x103c44_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_15b_unit::attack_secondary_body_block_15b;

mod attack_secondary_body_block_15c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_15c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103c44: move.w #$fd4a, -(a7)
        e.step()?;
        let value = 0xfd4a_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103c48: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103c4e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00103c4e: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x103c50_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_15c_unit::attack_secondary_body_block_15c;

mod attack_secondary_body_block_15d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_15d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103c50: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00103c54: bge.w $103d52
        e.step()?;
        pc = if r.greater_equal()? {
            0x103d52_u32
        } else {
            0x103c58_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_15d_unit::attack_secondary_body_block_15d;

mod attack_secondary_body_block_15e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_15e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103c58: tst.w -$d68(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff298_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00103c5c: beq.b $103ca8
        e.step()?;
        pc = if r.equal()? {
            0x103ca8_u32
        } else {
            0x103c5e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_15e_unit::attack_secondary_body_block_15e;

mod attack_secondary_body_block_15f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_15f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103c5e: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103c60: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103c64: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103c6a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103c6c: moveq #$ec, d0
        e.step()?;
        r.put_data(0, 32, 0xffffffec_u32);
        r.move_flags(0xffffffec_u32, 32);
        // 00103c6e: add.w -$d66(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff29a_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103c72: cmpi.w #$ffff, $5e(a0, d0.w * 2)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x5e_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00103c78: ble.b $103c98
        e.step()?;
        pc = if r.less_equal()? {
            0x103c98_u32
        } else {
            0x103c7a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_15f_unit::attack_secondary_body_block_15f;

mod attack_secondary_body_block_160_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_160(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103c7a: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103c7c: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103c80: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103c86: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103c88: moveq #$ec, d0
        e.step()?;
        r.put_data(0, 32, 0xffffffec_u32);
        r.move_flags(0xffffffec_u32, 32);
        // 00103c8a: add.w -$d66(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff29a_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103c8e: lea.l $5e(a0, d0.w * 2), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x5e_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.put_address(0, value);
        // 00103c92: move.w -$d6a(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff296_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103c96: add.w d0, (a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x103c98_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_160_unit::attack_secondary_body_block_160;

mod attack_secondary_body_block_161_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_161(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103c98: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103c9a: move.w #$ffd8, -(a7)
        e.step()?;
        let value = 0xffd8_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103c9e: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103ca0: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103ca6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00103ca6: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x103ca8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_161_unit::attack_secondary_body_block_161;

mod attack_secondary_body_block_162_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_162(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103ca8: move.w d7, d0
        e.step()?;
        let value = r.data(7, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103caa: add.w $4434b8.l, d0
        e.step()?;
        let source_address = 0x4434b8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103cb0: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103cb2: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00103cb4: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00103cb8: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103cbe: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103cc0: add.l d0, $a42(a0)
        e.step()?;
        let source = r.data(0, 32)?;
        let destination_address = r.address(0)?.wrapping_add(0xa42_u32);
        let destination = e.l(destination_address)?;
        let value = r.add_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 00103cc4: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103cc6: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103cca: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103cd0: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103cd2: addq.l #$1, $a4e(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0xa4e_u32);
        let destination = e.l(destination_address)?;
        let value = r.add_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 00103cd6: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103cd8: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103cdc: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103ce2: lea.l $50(a0, d0.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x50_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 00103ce6: sub.w d7, (a0)
        e.step()?;
        let source = r.data(7, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00103ce8: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103cea: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103cee: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103cf4: lea.l $50(a0, d0.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x50_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 00103cf8: move.w $4434b8.l, d0
        e.step()?;
        let value_address = 0x4434b8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103cfe: sub.w d0, (a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00103d00: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103d02: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103d06: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103d0c: cmpi.w #$1, $50(a0, d0.l)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x50_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00103d12: bge.b $103d3c
        e.step()?;
        pc = if r.greater_equal()? {
            0x103d3c_u32
        } else {
            0x103d14_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_162_unit::attack_secondary_body_block_162;

mod attack_secondary_body_block_163_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_163(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103d14: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103d16: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103d1a: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103d20: cmpi.w #$fff6, $50(a0, d0.l)
        e.step()?;
        let source = 0xfff6_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x50_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00103d26: ble.b $103d3c
        e.step()?;
        pc = if r.less_equal()? {
            0x103d3c_u32
        } else {
            0x103d28_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_163_unit::attack_secondary_body_block_163;

mod attack_secondary_body_block_164_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_164(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103d28: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103d2a: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103d2e: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103d34: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103d36: move.b #$1, $9de(a0)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x9de_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x103d3c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_164_unit::attack_secondary_body_block_164;

mod attack_secondary_body_block_165_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_165(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103d3c: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103d3e: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103d42: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103d48: move.w #$1, $14(a0, d0.l)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x14_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103d4e: bra.w $103dec
        e.step()?;
        pc = 0x103dec_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_165_unit::attack_secondary_body_block_165;

mod attack_secondary_body_block_166_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_166(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103d52: tst.w -$d68(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff298_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00103d56: beq.b $103da8
        e.step()?;
        pc = if r.equal()? {
            0x103da8_u32
        } else {
            0x103d58_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_166_unit::attack_secondary_body_block_166;

mod attack_secondary_body_block_167_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_167(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103d58: moveq #$ec, d0
        e.step()?;
        r.put_data(0, 32, 0xffffffec_u32);
        r.move_flags(0xffffffec_u32, 32);
        // 00103d5a: add.w -$d66(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff29a_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103d5e: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103d60: add.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00103d62: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00103d66: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103d6c: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103d6e: adda.w d0, a0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.address(0)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103d70: cmpi.b #$ff, $b2(a0)
        e.step()?;
        let source = 0xff_u32;
        let destination_address = r.address(0)?.wrapping_add(0xb2_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00103d76: ble.b $103d98
        e.step()?;
        pc = if r.less_equal()? {
            0x103d98_u32
        } else {
            0x103d78_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_167_unit::attack_secondary_body_block_167;

mod attack_secondary_body_block_168_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_168(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103d78: moveq #$ec, d0
        e.step()?;
        r.put_data(0, 32, 0xffffffec_u32);
        r.move_flags(0xffffffec_u32, 32);
        // 00103d7a: add.w -$d66(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xfffff29a_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103d7e: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103d80: add.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00103d82: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00103d86: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103d8c: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103d8e: adda.w d0, a0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.address(0)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103d90: move.b -$d69(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff297_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00103d94: add.b d0, $b2(a0)
        e.step()?;
        let source = r.data(0, 8)?;
        let destination_address = r.address(0)?.wrapping_add(0xb2_u32);
        let destination = u32::from(e.b(destination_address)?);
        let value = r.add_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        pc = 0x103d98_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_168_unit::attack_secondary_body_block_168;

mod attack_secondary_body_block_169_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_169(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103d98: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103d9a: move.w #$ffd8, -(a7)
        e.step()?;
        let value = 0xffd8_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103d9e: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103da0: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103da6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00103da6: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x103da8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_169_unit::attack_secondary_body_block_169;
mod attack_secondary_body_block_16a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_16a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103da8: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103daa: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103dac: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103db0: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103db6: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103db8: move.b #$1, $105(a0)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x105_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00103dbe: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103dc0: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103dc2: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103dc6: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103dcc: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103dce: move.w $4434b8.l, d0
        e.step()?;
        let value_address = 0x4434b8_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103dd4: sub.w d0, $110(a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x110_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00103dd8: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103dda: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103ddc: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103de0: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103de6: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103de8: sub.w d7, $110(a0)
        e.step()?;
        let source = r.data(7, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x110_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x103dec_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_16a_unit::attack_secondary_body_block_16a;

mod attack_secondary_body_block_16b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_16b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103dec: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103dee: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103df0: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103df2: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103df8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00103df8: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103dfa: bra.b $103e36
        e.step()?;
        pc = 0x103e36_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_16b_unit::attack_secondary_body_block_16b;

mod attack_secondary_body_block_16c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_16c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103dfc: cmpi.w #$6, d3
        e.step()?;
        let source = 0x6_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00103e00: bge.b $103e14
        e.step()?;
        pc = if r.greater_equal()? {
            0x103e14_u32
        } else {
            0x103e02_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_16c_unit::attack_secondary_body_block_16c;

mod attack_secondary_body_block_16d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_16d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103e02: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103e04: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103e08: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103e0e: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103e10: addq.l #$1, $a56(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0xa56_u32);
        let destination = e.l(destination_address)?;
        let value = r.add_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        pc = 0x103e14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_16d_unit::attack_secondary_body_block_16d;

mod attack_secondary_body_block_16e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_16e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103e14: clr.w $442bc8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103e1a: clr.w $4434b8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103e20: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00103e22: move.b d0, -$d63(a6)
        e.step()?;
        let value = r.data(0, 8)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffff29d_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00103e26: move.w d4, -(a7)
        e.step()?;
        let value = r.data(4, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103e28: move.w #$ffff, -(a7)
        e.step()?;
        let value = 0xffff_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103e2c: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103e2e: bsr.l $158454
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103e34_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x158454_u32)?;
        // 00103e34: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x103e36_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_16e_unit::attack_secondary_body_block_16e;

mod attack_secondary_body_block_16f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_16f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103e36: cmpi.w #$9, d3
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00103e3a: bge.b $103e64
        e.step()?;
        pc = if r.greater_equal()? {
            0x103e64_u32
        } else {
            0x103e3c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_16f_unit::attack_secondary_body_block_16f;

mod attack_secondary_body_block_170_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_170(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103e3c: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00103e3e: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103e42: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00103e48: cmpi.w #$1, $50(a0, d0.l)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x50_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00103e4e: bge.b $103e8e
        e.step()?;
        pc = if r.greater_equal()? {
            0x103e8e_u32
        } else {
            0x103e50_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_170_unit::attack_secondary_body_block_170;

mod attack_secondary_body_block_171_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_171(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103e50: move.b #$1, -$d6b(a6)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff295_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00103e56: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103e58: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103e5a: bsr.l $12e4e4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103e60_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x12e4e4_u32)?;
        // 00103e60: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00103e62: bra.b $103e8e
        e.step()?;
        pc = 0x103e8e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_171_unit::attack_secondary_body_block_171;

mod attack_secondary_body_block_172_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_172(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103e64: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00103e66: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00103e68: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00103e6c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00103e72: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00103e74: cmpi.w #$1, $110(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x110_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00103e7a: bge.b $103e8e
        e.step()?;
        pc = if r.greater_equal()? {
            0x103e8e_u32
        } else {
            0x103e7c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_172_unit::attack_secondary_body_block_172;

mod attack_secondary_body_block_173_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_173(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103e7c: move.b #$1, -$d6b(a6)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffff295_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00103e82: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103e84: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103e86: bsr.l $12e4e4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103e8c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x12e4e4_u32)?;
        // 00103e8c: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x103e8e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_173_unit::attack_secondary_body_block_173;

mod attack_secondary_body_block_174_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_174(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103e8e: clr.w $442bc8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442bc8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103e94: clr.w $4434b8.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4434b8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103e9a: move.w #$f, -(a7)
        e.step()?;
        let value = 0xf_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00103e9e: bsr.l $13154c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x103ea4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13154c_u32)?;
        // 00103ea4: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00103ea6: move.b -$d6b(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffff295_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00103eaa: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x103eac_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_174_unit::attack_secondary_body_block_174;

mod attack_secondary_body_block_175_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_175(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00103eac: movem.l (a7)+, d3-d7
        e.step()?;
        r.restore_register(e, 3)?;
        r.restore_register(e, 4)?;
        r.restore_register(e, 5)?;
        r.restore_register(e, 6)?;
        r.restore_register(e, 7)?;
        // 00103eb0: unlk a6
        e.step()?;
        r.put_address(7, r.address(6)?);
        r.restore_register(e, 14)?;
        // 00103eb2: rts
        e.step()?;
        return finish(e, r).map(|_| std::ops::ControlFlow::Break(()));

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_175_unit::attack_secondary_body_block_175;

mod attack_secondary_body_block_176_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn attack_secondary_body_block_176(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        return Err(RuleError::Invalid(format!(
            "source transition outside recovered body: {pc:08x}"
        ))
        .into());

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use attack_secondary_body_block_176_unit::attack_secondary_body_block_176;
