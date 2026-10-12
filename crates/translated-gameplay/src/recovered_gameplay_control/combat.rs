//! Complete provisional gameplay control from canonical CODE 1 bodies.

use crate::RuleError;
use crate::recovered_application_control::{Execution, Result, Signal};
use crate::recovered_native_services::NativeCall;
use crate::recovered_system_control::{child, cpu, finish, imported};

pub fn combat_body(e: &mut Execution<'_>, frame: u32) -> Result<()> {
    let mut r = cpu(e, frame);
    let result = (|| {
        let mut pc = 0x114378_u32;
        loop {
            match match pc {
                0x114378 => combat_body_block_0(e, &mut r, frame, pc)?,
                0x1143c0 => combat_body_block_1(e, &mut r, frame, pc)?,
                0x1143c6 => combat_body_block_2(e, &mut r, frame, pc)?,
                0x1143ee => combat_body_block_3(e, &mut r, frame, pc)?,
                0x1143fa => combat_body_block_4(e, &mut r, frame, pc)?,
                0x114442 => combat_body_block_5(e, &mut r, frame, pc)?,
                0x11445a => combat_body_block_6(e, &mut r, frame, pc)?,
                0x114484 => combat_body_block_7(e, &mut r, frame, pc)?,
                0x114498 => combat_body_block_8(e, &mut r, frame, pc)?,
                0x1144a8 => combat_body_block_9(e, &mut r, frame, pc)?,
                0x1144aa => combat_body_block_a(e, &mut r, frame, pc)?,
                0x1144b6 => combat_body_block_b(e, &mut r, frame, pc)?,
                0x1144be => combat_body_block_c(e, &mut r, frame, pc)?,
                0x1144c2 => combat_body_block_d(e, &mut r, frame, pc)?,
                0x1144c4 => combat_body_block_e(e, &mut r, frame, pc)?,
                0x1144e0 => combat_body_block_f(e, &mut r, frame, pc)?,
                0x1144fa => combat_body_block_10(e, &mut r, frame, pc)?,
                0x114548 => combat_body_block_11(e, &mut r, frame, pc)?,
                0x114556 => combat_body_block_12(e, &mut r, frame, pc)?,
                0x114570 => combat_body_block_13(e, &mut r, frame, pc)?,
                0x114586 => combat_body_block_14(e, &mut r, frame, pc)?,
                0x11458e => combat_body_block_15(e, &mut r, frame, pc)?,
                0x1145a2 => combat_body_block_16(e, &mut r, frame, pc)?,
                0x1145b4 => combat_body_block_17(e, &mut r, frame, pc)?,
                0x1145be => combat_body_block_18(e, &mut r, frame, pc)?,
                0x1145c8 => combat_body_block_19(e, &mut r, frame, pc)?,
                0x1145e4 => combat_body_block_1a(e, &mut r, frame, pc)?,
                0x1145f4 => combat_body_block_1b(e, &mut r, frame, pc)?,
                0x114624 => combat_body_block_1c(e, &mut r, frame, pc)?,
                0x114626 => combat_body_block_1d(e, &mut r, frame, pc)?,
                0x114630 => combat_body_block_1e(e, &mut r, frame, pc)?,
                0x114632 => combat_body_block_1f(e, &mut r, frame, pc)?,
                0x114658 => combat_body_block_20(e, &mut r, frame, pc)?,
                0x11465e => combat_body_block_21(e, &mut r, frame, pc)?,
                0x11467a => combat_body_block_22(e, &mut r, frame, pc)?,
                0x114694 => combat_body_block_23(e, &mut r, frame, pc)?,
                0x1146aa => combat_body_block_24(e, &mut r, frame, pc)?,
                0x1146c4 => combat_body_block_25(e, &mut r, frame, pc)?,
                0x1146ee => combat_body_block_26(e, &mut r, frame, pc)?,
                0x114738 => combat_body_block_27(e, &mut r, frame, pc)?,
                0x114756 => combat_body_block_28(e, &mut r, frame, pc)?,
                0x114760 => combat_body_block_29(e, &mut r, frame, pc)?,
                0x11476a => combat_body_block_2a(e, &mut r, frame, pc)?,
                0x114784 => combat_body_block_2b(e, &mut r, frame, pc)?,
                0x114794 => combat_body_block_2c(e, &mut r, frame, pc)?,
                0x1147be => combat_body_block_2d(e, &mut r, frame, pc)?,
                0x1147d8 => combat_body_block_2e(e, &mut r, frame, pc)?,
                0x1147f2 => combat_body_block_2f(e, &mut r, frame, pc)?,
                0x11480c => combat_body_block_30(e, &mut r, frame, pc)?,
                0x114826 => combat_body_block_31(e, &mut r, frame, pc)?,
                0x114830 => combat_body_block_32(e, &mut r, frame, pc)?,
                0x11484a => combat_body_block_33(e, &mut r, frame, pc)?,
                0x114852 => combat_body_block_34(e, &mut r, frame, pc)?,
                0x114856 => combat_body_block_35(e, &mut r, frame, pc)?,
                0x114886 => combat_body_block_36(e, &mut r, frame, pc)?,
                0x1148ac => combat_body_block_37(e, &mut r, frame, pc)?,
                0x1148ae => combat_body_block_38(e, &mut r, frame, pc)?,
                0x1148b4 => combat_body_block_39(e, &mut r, frame, pc)?,
                0x1148b8 => combat_body_block_3a(e, &mut r, frame, pc)?,
                0x1148bc => combat_body_block_3b(e, &mut r, frame, pc)?,
                0x1148c8 => combat_body_block_3c(e, &mut r, frame, pc)?,
                0x1148ce => combat_body_block_3d(e, &mut r, frame, pc)?,
                0x1148da => combat_body_block_3e(e, &mut r, frame, pc)?,
                0x1148de => combat_body_block_3f(e, &mut r, frame, pc)?,
                0x1148f2 => combat_body_block_40(e, &mut r, frame, pc)?,
                0x1148fe => combat_body_block_41(e, &mut r, frame, pc)?,
                0x114900 => combat_body_block_42(e, &mut r, frame, pc)?,
                0x11490c => combat_body_block_43(e, &mut r, frame, pc)?,
                0x114910 => combat_body_block_44(e, &mut r, frame, pc)?,
                0x114924 => combat_body_block_45(e, &mut r, frame, pc)?,
                0x114934 => combat_body_block_46(e, &mut r, frame, pc)?,
                0x114936 => combat_body_block_47(e, &mut r, frame, pc)?,
                0x11493e => combat_body_block_48(e, &mut r, frame, pc)?,
                0x114970 => combat_body_block_49(e, &mut r, frame, pc)?,
                0x11497c => combat_body_block_4a(e, &mut r, frame, pc)?,
                0x114980 => combat_body_block_4b(e, &mut r, frame, pc)?,
                0x1149a6 => combat_body_block_4c(e, &mut r, frame, pc)?,
                0x1149ba => combat_body_block_4d(e, &mut r, frame, pc)?,
                0x1149c6 => combat_body_block_4e(e, &mut r, frame, pc)?,
                0x1149c8 => combat_body_block_4f(e, &mut r, frame, pc)?,
                0x1149d4 => combat_body_block_50(e, &mut r, frame, pc)?,
                0x1149d8 => combat_body_block_51(e, &mut r, frame, pc)?,
                0x1149fe => combat_body_block_52(e, &mut r, frame, pc)?,
                0x114a12 => combat_body_block_53(e, &mut r, frame, pc)?,
                0x114a22 => combat_body_block_54(e, &mut r, frame, pc)?,
                0x114a24 => combat_body_block_55(e, &mut r, frame, pc)?,
                0x114a2c => combat_body_block_56(e, &mut r, frame, pc)?,
                0x114a74 => combat_body_block_57(e, &mut r, frame, pc)?,
                0x114a80 => combat_body_block_58(e, &mut r, frame, pc)?,
                0x114a84 => combat_body_block_59(e, &mut r, frame, pc)?,
                0x114aaa => combat_body_block_5a(e, &mut r, frame, pc)?,
                0x114abe => combat_body_block_5b(e, &mut r, frame, pc)?,
                0x114aca => combat_body_block_5c(e, &mut r, frame, pc)?,
                0x114acc => combat_body_block_5d(e, &mut r, frame, pc)?,
                0x114ad8 => combat_body_block_5e(e, &mut r, frame, pc)?,
                0x114adc => combat_body_block_5f(e, &mut r, frame, pc)?,
                0x114b02 => combat_body_block_60(e, &mut r, frame, pc)?,
                0x114b16 => combat_body_block_61(e, &mut r, frame, pc)?,
                0x114b26 => combat_body_block_62(e, &mut r, frame, pc)?,
                0x114b28 => combat_body_block_63(e, &mut r, frame, pc)?,
                0x114b30 => combat_body_block_64(e, &mut r, frame, pc)?,
                0x114b78 => combat_body_block_65(e, &mut r, frame, pc)?,
                0x114ba0 => combat_body_block_66(e, &mut r, frame, pc)?,
                0x114baa => combat_body_block_67(e, &mut r, frame, pc)?,
                0x114be8 => combat_body_block_68(e, &mut r, frame, pc)?,
                0x114bee => combat_body_block_69(e, &mut r, frame, pc)?,
                0x114c0e => combat_body_block_6a(e, &mut r, frame, pc)?,
                0x114c14 => combat_body_block_6b(e, &mut r, frame, pc)?,
                0x114c50 => combat_body_block_6c(e, &mut r, frame, pc)?,
                0x114c58 => combat_body_block_6d(e, &mut r, frame, pc)?,
                0x114c6e => combat_body_block_6e(e, &mut r, frame, pc)?,
                0x114c78 => combat_body_block_6f(e, &mut r, frame, pc)?,
                0x114c80 => combat_body_block_70(e, &mut r, frame, pc)?,
                0x114ca6 => combat_body_block_71(e, &mut r, frame, pc)?,
                0x114cb6 => combat_body_block_72(e, &mut r, frame, pc)?,
                0x114cb8 => combat_body_block_73(e, &mut r, frame, pc)?,
                0x114cd2 => combat_body_block_74(e, &mut r, frame, pc)?,
                0x114cdc => combat_body_block_75(e, &mut r, frame, pc)?,
                0x114cea => combat_body_block_76(e, &mut r, frame, pc)?,
                0x114cf6 => combat_body_block_77(e, &mut r, frame, pc)?,
                0x114d02 => combat_body_block_78(e, &mut r, frame, pc)?,
                0x114d12 => combat_body_block_79(e, &mut r, frame, pc)?,
                0x114d18 => combat_body_block_7a(e, &mut r, frame, pc)?,
                0x114d42 => combat_body_block_7b(e, &mut r, frame, pc)?,
                0x114d48 => combat_body_block_7c(e, &mut r, frame, pc)?,
                0x114d84 => combat_body_block_7d(e, &mut r, frame, pc)?,
                0x114dc8 => combat_body_block_7e(e, &mut r, frame, pc)?,
                0x114dca => combat_body_block_7f(e, &mut r, frame, pc)?,
                0x114dd2 => combat_body_block_80(e, &mut r, frame, pc)?,
                0x114dd6 => combat_body_block_81(e, &mut r, frame, pc)?,
                0x114de2 => combat_body_block_82(e, &mut r, frame, pc)?,
                0x114df2 => combat_body_block_83(e, &mut r, frame, pc)?,
                0x114e20 => combat_body_block_84(e, &mut r, frame, pc)?,
                0x114e26 => combat_body_block_85(e, &mut r, frame, pc)?,
                0x114e2a => combat_body_block_86(e, &mut r, frame, pc)?,
                0x114e7c => combat_body_block_87(e, &mut r, frame, pc)?,
                0x114e84 => combat_body_block_88(e, &mut r, frame, pc)?,
                0x114e88 => combat_body_block_89(e, &mut r, frame, pc)?,
                0x114e8c => combat_body_block_8a(e, &mut r, frame, pc)?,
                0x114ee6 => combat_body_block_8b(e, &mut r, frame, pc)?,
                0x114eee => combat_body_block_8c(e, &mut r, frame, pc)?,
                0x114ef2 => combat_body_block_8d(e, &mut r, frame, pc)?,
                0x114efe => combat_body_block_8e(e, &mut r, frame, pc)?,
                0x114f0e => combat_body_block_8f(e, &mut r, frame, pc)?,
                0x114f18 => combat_body_block_90(e, &mut r, frame, pc)?,
                0x114f20 => combat_body_block_91(e, &mut r, frame, pc)?,
                0x114f2c => combat_body_block_92(e, &mut r, frame, pc)?,
                0x114f30 => combat_body_block_93(e, &mut r, frame, pc)?,
                0x114f5c => combat_body_block_94(e, &mut r, frame, pc)?,
                0x114f78 => combat_body_block_95(e, &mut r, frame, pc)?,
                0x114f90 => combat_body_block_96(e, &mut r, frame, pc)?,
                0x114fa6 => combat_body_block_97(e, &mut r, frame, pc)?,
                0x114fb2 => combat_body_block_98(e, &mut r, frame, pc)?,
                0x114fba => combat_body_block_99(e, &mut r, frame, pc)?,
                0x115000 => combat_body_block_9a(e, &mut r, frame, pc)?,
                0x115018 => combat_body_block_9b(e, &mut r, frame, pc)?,
                0x115030 => combat_body_block_9c(e, &mut r, frame, pc)?,
                0x115078 => combat_body_block_9d(e, &mut r, frame, pc)?,
                0x115098 => combat_body_block_9e(e, &mut r, frame, pc)?,
                0x1150ac => combat_body_block_9f(e, &mut r, frame, pc)?,
                0x1150c0 => combat_body_block_a0(e, &mut r, frame, pc)?,
                0x1150d8 => combat_body_block_a1(e, &mut r, frame, pc)?,
                0x1150e4 => combat_body_block_a2(e, &mut r, frame, pc)?,
                0x1150ee => combat_body_block_a3(e, &mut r, frame, pc)?,
                0x11510a => combat_body_block_a4(e, &mut r, frame, pc)?,
                0x11511e => combat_body_block_a5(e, &mut r, frame, pc)?,
                0x115136 => combat_body_block_a6(e, &mut r, frame, pc)?,
                0x115140 => combat_body_block_a7(e, &mut r, frame, pc)?,
                0x115156 => combat_body_block_a8(e, &mut r, frame, pc)?,
                0x11515a => combat_body_block_a9(e, &mut r, frame, pc)?,
                0x11516c => combat_body_block_aa(e, &mut r, frame, pc)?,
                0x115188 => combat_body_block_ab(e, &mut r, frame, pc)?,
                0x11518e => combat_body_block_ac(e, &mut r, frame, pc)?,
                0x1151a2 => combat_body_block_ad(e, &mut r, frame, pc)?,
                0x1151c8 => combat_body_block_ae(e, &mut r, frame, pc)?,
                0x1151ce => combat_body_block_af(e, &mut r, frame, pc)?,
                0x1151e6 => combat_body_block_b0(e, &mut r, frame, pc)?,
                0x11520e => combat_body_block_b1(e, &mut r, frame, pc)?,
                0x115214 => combat_body_block_b2(e, &mut r, frame, pc)?,
                0x11521c => combat_body_block_b3(e, &mut r, frame, pc)?,
                0x115232 => combat_body_block_b4(e, &mut r, frame, pc)?,
                0x11523e => combat_body_block_b5(e, &mut r, frame, pc)?,
                0x115244 => combat_body_block_b6(e, &mut r, frame, pc)?,
                0x11524a => combat_body_block_b7(e, &mut r, frame, pc)?,
                0x11526c => combat_body_block_b8(e, &mut r, frame, pc)?,
                0x115274 => combat_body_block_b9(e, &mut r, frame, pc)?,
                0x11527c => combat_body_block_ba(e, &mut r, frame, pc)?,
                0x1152a2 => combat_body_block_bb(e, &mut r, frame, pc)?,
                0x1152aa => combat_body_block_bc(e, &mut r, frame, pc)?,
                0x1152ce => combat_body_block_bd(e, &mut r, frame, pc)?,
                0x1152e4 => combat_body_block_be(e, &mut r, frame, pc)?,
                0x1152ec => combat_body_block_bf(e, &mut r, frame, pc)?,
                0x115310 => combat_body_block_c0(e, &mut r, frame, pc)?,
                0x115318 => combat_body_block_c1(e, &mut r, frame, pc)?,
                0x115340 => combat_body_block_c2(e, &mut r, frame, pc)?,
                0x115348 => combat_body_block_c3(e, &mut r, frame, pc)?,
                0x11536c => combat_body_block_c4(e, &mut r, frame, pc)?,
                0x115384 => combat_body_block_c5(e, &mut r, frame, pc)?,
                0x11538c => combat_body_block_c6(e, &mut r, frame, pc)?,
                0x1153b0 => combat_body_block_c7(e, &mut r, frame, pc)?,
                0x1153c6 => combat_body_block_c8(e, &mut r, frame, pc)?,
                0x1153ce => combat_body_block_c9(e, &mut r, frame, pc)?,
                0x1153f2 => combat_body_block_ca(e, &mut r, frame, pc)?,
                0x115408 => combat_body_block_cb(e, &mut r, frame, pc)?,
                0x115446 => combat_body_block_cc(e, &mut r, frame, pc)?,
                0x115460 => combat_body_block_cd(e, &mut r, frame, pc)?,
                0x115480 => combat_body_block_ce(e, &mut r, frame, pc)?,
                0x11549c => combat_body_block_cf(e, &mut r, frame, pc)?,
                0x1154a4 => combat_body_block_d0(e, &mut r, frame, pc)?,
                0x1154d4 => combat_body_block_d1(e, &mut r, frame, pc)?,
                0x1154f0 => combat_body_block_d2(e, &mut r, frame, pc)?,
                0x11550e => combat_body_block_d3(e, &mut r, frame, pc)?,
                0x115526 => combat_body_block_d4(e, &mut r, frame, pc)?,
                0x115532 => combat_body_block_d5(e, &mut r, frame, pc)?,
                0x115548 => combat_body_block_d6(e, &mut r, frame, pc)?,
                0x115560 => combat_body_block_d7(e, &mut r, frame, pc)?,
                0x11556a => combat_body_block_d8(e, &mut r, frame, pc)?,
                0x1155a0 => combat_body_block_d9(e, &mut r, frame, pc)?,
                0x1155ba => combat_body_block_da(e, &mut r, frame, pc)?,
                0x1155d4 => combat_body_block_db(e, &mut r, frame, pc)?,
                0x1155ee => combat_body_block_dc(e, &mut r, frame, pc)?,
                0x115608 => combat_body_block_dd(e, &mut r, frame, pc)?,
                0x11560c => combat_body_block_de(e, &mut r, frame, pc)?,
                0x115626 => combat_body_block_df(e, &mut r, frame, pc)?,
                0x115642 => combat_body_block_e0(e, &mut r, frame, pc)?,
                0x11564c => combat_body_block_e1(e, &mut r, frame, pc)?,
                0x115664 => combat_body_block_e2(e, &mut r, frame, pc)?,
                0x11567c => combat_body_block_e3(e, &mut r, frame, pc)?,
                0x115696 => combat_body_block_e4(e, &mut r, frame, pc)?,
                0x1156b0 => combat_body_block_e5(e, &mut r, frame, pc)?,
                0x1156c8 => combat_body_block_e6(e, &mut r, frame, pc)?,
                0x1156d2 => combat_body_block_e7(e, &mut r, frame, pc)?,
                0x1156dc => combat_body_block_e8(e, &mut r, frame, pc)?,
                0x1156f2 => combat_body_block_e9(e, &mut r, frame, pc)?,
                0x1156fa => combat_body_block_ea(e, &mut r, frame, pc)?,
                0x115732 => combat_body_block_eb(e, &mut r, frame, pc)?,
                0x115748 => combat_body_block_ec(e, &mut r, frame, pc)?,
                0x115762 => combat_body_block_ed(e, &mut r, frame, pc)?,
                0x11576c => combat_body_block_ee(e, &mut r, frame, pc)?,
                0x115776 => combat_body_block_ef(e, &mut r, frame, pc)?,
                0x11578a => combat_body_block_f0(e, &mut r, frame, pc)?,
                0x115792 => combat_body_block_f1(e, &mut r, frame, pc)?,
                0x1157aa => combat_body_block_f2(e, &mut r, frame, pc)?,
                0x1157c0 => combat_body_block_f3(e, &mut r, frame, pc)?,
                0x1157d4 => combat_body_block_f4(e, &mut r, frame, pc)?,
                0x1157e8 => combat_body_block_f5(e, &mut r, frame, pc)?,
                0x115816 => combat_body_block_f6(e, &mut r, frame, pc)?,
                0x11582c => combat_body_block_f7(e, &mut r, frame, pc)?,
                0x115842 => combat_body_block_f8(e, &mut r, frame, pc)?,
                0x115860 => combat_body_block_f9(e, &mut r, frame, pc)?,
                0x115882 => combat_body_block_fa(e, &mut r, frame, pc)?,
                0x1158a6 => combat_body_block_fb(e, &mut r, frame, pc)?,
                0x1158c8 => combat_body_block_fc(e, &mut r, frame, pc)?,
                0x1158ec => combat_body_block_fd(e, &mut r, frame, pc)?,
                0x1158f4 => combat_body_block_fe(e, &mut r, frame, pc)?,
                0x115952 => combat_body_block_ff(e, &mut r, frame, pc)?,
                0x11597c => combat_body_block_100(e, &mut r, frame, pc)?,
                0x115992 => combat_body_block_101(e, &mut r, frame, pc)?,
                0x11599e => combat_body_block_102(e, &mut r, frame, pc)?,
                0x1159cc => combat_body_block_103(e, &mut r, frame, pc)?,
                0x1159f6 => combat_body_block_104(e, &mut r, frame, pc)?,
                0x1159fa => combat_body_block_105(e, &mut r, frame, pc)?,
                0x115a06 => combat_body_block_106(e, &mut r, frame, pc)?,
                0x115a0a => combat_body_block_107(e, &mut r, frame, pc)?,
                0x115a0e => combat_body_block_108(e, &mut r, frame, pc)?,
                0x115a12 => combat_body_block_109(e, &mut r, frame, pc)?,
                0x115a1a => combat_body_block_10a(e, &mut r, frame, pc)?,
                0x115a1e => combat_body_block_10b(e, &mut r, frame, pc)?,
                0x115a58 => combat_body_block_10c(e, &mut r, frame, pc)?,
                0x115a6c => combat_body_block_10d(e, &mut r, frame, pc)?,
                0x115a82 => combat_body_block_10e(e, &mut r, frame, pc)?,
                0x115abe => combat_body_block_10f(e, &mut r, frame, pc)?,
                0x115ac4 => combat_body_block_110(e, &mut r, frame, pc)?,
                0x115ae4 => combat_body_block_111(e, &mut r, frame, pc)?,
                0x115afa => combat_body_block_112(e, &mut r, frame, pc)?,
                0x115b50 => combat_body_block_113(e, &mut r, frame, pc)?,
                0x115b72 => combat_body_block_114(e, &mut r, frame, pc)?,
                0x115b82 => combat_body_block_115(e, &mut r, frame, pc)?,
                0x115b90 => combat_body_block_116(e, &mut r, frame, pc)?,
                0x115b98 => combat_body_block_117(e, &mut r, frame, pc)?,
                0x115ba0 => combat_body_block_118(e, &mut r, frame, pc)?,
                0x115ba8 => combat_body_block_119(e, &mut r, frame, pc)?,
                0x115bb0 => combat_body_block_11a(e, &mut r, frame, pc)?,
                0x115bb8 => combat_body_block_11b(e, &mut r, frame, pc)?,
                0x115bc0 => combat_body_block_11c(e, &mut r, frame, pc)?,
                0x115bc8 => combat_body_block_11d(e, &mut r, frame, pc)?,
                0x115bd0 => combat_body_block_11e(e, &mut r, frame, pc)?,
                0x115bd8 => combat_body_block_11f(e, &mut r, frame, pc)?,
                0x115be0 => combat_body_block_120(e, &mut r, frame, pc)?,
                0x115be6 => combat_body_block_121(e, &mut r, frame, pc)?,
                0x115bee => combat_body_block_122(e, &mut r, frame, pc)?,
                0x115bf6 => combat_body_block_123(e, &mut r, frame, pc)?,
                0x115bfe => combat_body_block_124(e, &mut r, frame, pc)?,
                0x115c06 => combat_body_block_125(e, &mut r, frame, pc)?,
                0x115c0e => combat_body_block_126(e, &mut r, frame, pc)?,
                0x115c16 => combat_body_block_127(e, &mut r, frame, pc)?,
                0x115c1e => combat_body_block_128(e, &mut r, frame, pc)?,
                0x115c26 => combat_body_block_129(e, &mut r, frame, pc)?,
                0x115c2e => combat_body_block_12a(e, &mut r, frame, pc)?,
                0x115c36 => combat_body_block_12b(e, &mut r, frame, pc)?,
                0x115c3e => combat_body_block_12c(e, &mut r, frame, pc)?,
                0x115c46 => combat_body_block_12d(e, &mut r, frame, pc)?,
                0x115c4e => combat_body_block_12e(e, &mut r, frame, pc)?,
                0x115c56 => combat_body_block_12f(e, &mut r, frame, pc)?,
                0x115c5e => combat_body_block_130(e, &mut r, frame, pc)?,
                0x115c62 => combat_body_block_131(e, &mut r, frame, pc)?,
                0x115c6e => combat_body_block_132(e, &mut r, frame, pc)?,
                0x115c7a => combat_body_block_133(e, &mut r, frame, pc)?,
                0x115c86 => combat_body_block_134(e, &mut r, frame, pc)?,
                0x115c92 => combat_body_block_135(e, &mut r, frame, pc)?,
                0x115c9e => combat_body_block_136(e, &mut r, frame, pc)?,
                0x115caa => combat_body_block_137(e, &mut r, frame, pc)?,
                0x115cb6 => combat_body_block_138(e, &mut r, frame, pc)?,
                0x115cc2 => combat_body_block_139(e, &mut r, frame, pc)?,
                0x115cce => combat_body_block_13a(e, &mut r, frame, pc)?,
                0x115cda => combat_body_block_13b(e, &mut r, frame, pc)?,
                0x115ce8 => combat_body_block_13c(e, &mut r, frame, pc)?,
                0x115cf6 => combat_body_block_13d(e, &mut r, frame, pc)?,
                0x115d04 => combat_body_block_13e(e, &mut r, frame, pc)?,
                0x115d12 => combat_body_block_13f(e, &mut r, frame, pc)?,
                0x115d6e => combat_body_block_140(e, &mut r, frame, pc)?,
                0x115dae => combat_body_block_141(e, &mut r, frame, pc)?,
                0x115dc6 => combat_body_block_142(e, &mut r, frame, pc)?,
                0x115dde => combat_body_block_143(e, &mut r, frame, pc)?,
                0x115df8 => combat_body_block_144(e, &mut r, frame, pc)?,
                0x115f0e => combat_body_block_145(e, &mut r, frame, pc)?,
                0x115f28 => combat_body_block_146(e, &mut r, frame, pc)?,
                0x115f92 => combat_body_block_147(e, &mut r, frame, pc)?,
                0x115fac => combat_body_block_148(e, &mut r, frame, pc)?,
                0x115fc2 => combat_body_block_149(e, &mut r, frame, pc)?,
                0x115fd8 => combat_body_block_14a(e, &mut r, frame, pc)?,
                0x115ff0 => combat_body_block_14b(e, &mut r, frame, pc)?,
                0x116006 => combat_body_block_14c(e, &mut r, frame, pc)?,
                0x11601c => combat_body_block_14d(e, &mut r, frame, pc)?,
                0x116036 => combat_body_block_14e(e, &mut r, frame, pc)?,
                0x116050 => combat_body_block_14f(e, &mut r, frame, pc)?,
                0x116102 => combat_body_block_150(e, &mut r, frame, pc)?,
                0x116116 => combat_body_block_151(e, &mut r, frame, pc)?,
                0x116128 => combat_body_block_152(e, &mut r, frame, pc)?,
                0x116130 => combat_body_block_153(e, &mut r, frame, pc)?,
                0x116132 => combat_body_block_154(e, &mut r, frame, pc)?,
                0x11613e => combat_body_block_155(e, &mut r, frame, pc)?,
                0x116146 => combat_body_block_156(e, &mut r, frame, pc)?,
                0x116158 => combat_body_block_157(e, &mut r, frame, pc)?,
                0x116166 => combat_body_block_158(e, &mut r, frame, pc)?,
                0x1161e4 => combat_body_block_159(e, &mut r, frame, pc)?,
                0x11623e => combat_body_block_15a(e, &mut r, frame, pc)?,
                0x116258 => combat_body_block_15b(e, &mut r, frame, pc)?,
                0x1162b4 => combat_body_block_15c(e, &mut r, frame, pc)?,
                0x1162c0 => combat_body_block_15d(e, &mut r, frame, pc)?,
                0x1162e2 => combat_body_block_15e(e, &mut r, frame, pc)?,
                0x1162e4 => combat_body_block_15f(e, &mut r, frame, pc)?,
                0x1162f0 => combat_body_block_160(e, &mut r, frame, pc)?,
                0x1162fe => combat_body_block_161(e, &mut r, frame, pc)?,
                0x116384 => combat_body_block_162(e, &mut r, frame, pc)?,
                0x1163b0 => combat_body_block_163(e, &mut r, frame, pc)?,
                0x11645c => combat_body_block_164(e, &mut r, frame, pc)?,
                0x116496 => combat_body_block_165(e, &mut r, frame, pc)?,
                0x1164aa => combat_body_block_166(e, &mut r, frame, pc)?,
                0x1164c8 => combat_body_block_167(e, &mut r, frame, pc)?,
                0x1164ce => combat_body_block_168(e, &mut r, frame, pc)?,
                0x1164d8 => combat_body_block_169(e, &mut r, frame, pc)?,
                0x116506 => combat_body_block_16a(e, &mut r, frame, pc)?,
                0x11651e => combat_body_block_16b(e, &mut r, frame, pc)?,
                0x116560 => combat_body_block_16c(e, &mut r, frame, pc)?,
                0x1165ea => combat_body_block_16d(e, &mut r, frame, pc)?,
                0x116674 => combat_body_block_16e(e, &mut r, frame, pc)?,
                0x116710 => combat_body_block_16f(e, &mut r, frame, pc)?,
                0x116794 => combat_body_block_170(e, &mut r, frame, pc)?,
                0x1167f2 => combat_body_block_171(e, &mut r, frame, pc)?,
                0x1167fc => combat_body_block_172(e, &mut r, frame, pc)?,
                0x116806 => combat_body_block_173(e, &mut r, frame, pc)?,
                0x116812 => combat_body_block_174(e, &mut r, frame, pc)?,
                0x1168d2 => combat_body_block_175(e, &mut r, frame, pc)?,
                0x1168ea => combat_body_block_176(e, &mut r, frame, pc)?,
                0x11691e => combat_body_block_177(e, &mut r, frame, pc)?,
                0x11692c => combat_body_block_178(e, &mut r, frame, pc)?,
                0x116934 => combat_body_block_179(e, &mut r, frame, pc)?,
                0x11693c => combat_body_block_17a(e, &mut r, frame, pc)?,
                0x11694c => combat_body_block_17b(e, &mut r, frame, pc)?,
                0x116954 => combat_body_block_17c(e, &mut r, frame, pc)?,
                0x11695c => combat_body_block_17d(e, &mut r, frame, pc)?,
                0x11696c => combat_body_block_17e(e, &mut r, frame, pc)?,
                0x116974 => combat_body_block_17f(e, &mut r, frame, pc)?,
                0x116984 => combat_body_block_180(e, &mut r, frame, pc)?,
                0x1169f8 => combat_body_block_181(e, &mut r, frame, pc)?,
                0x116a0e => combat_body_block_182(e, &mut r, frame, pc)?,
                0x116a18 => combat_body_block_183(e, &mut r, frame, pc)?,
                0x116a2e => combat_body_block_184(e, &mut r, frame, pc)?,
                0x116a36 => combat_body_block_185(e, &mut r, frame, pc)?,
                0x116a60 => combat_body_block_186(e, &mut r, frame, pc)?,
                0x116aea => combat_body_block_187(e, &mut r, frame, pc)?,
                0x116b18 => combat_body_block_188(e, &mut r, frame, pc)?,
                0x116b2c => combat_body_block_189(e, &mut r, frame, pc)?,
                0x116b36 => combat_body_block_18a(e, &mut r, frame, pc)?,
                0x116b4c => combat_body_block_18b(e, &mut r, frame, pc)?,
                0x116b5a => combat_body_block_18c(e, &mut r, frame, pc)?,
                0x116b64 => combat_body_block_18d(e, &mut r, frame, pc)?,
                0x116b84 => combat_body_block_18e(e, &mut r, frame, pc)?,
                0x116b92 => combat_body_block_18f(e, &mut r, frame, pc)?,
                0x116b9c => combat_body_block_190(e, &mut r, frame, pc)?,
                0x116bb0 => combat_body_block_191(e, &mut r, frame, pc)?,
                0x116bc2 => combat_body_block_192(e, &mut r, frame, pc)?,
                0x116be2 => combat_body_block_193(e, &mut r, frame, pc)?,
                0x116be6 => combat_body_block_194(e, &mut r, frame, pc)?,
                0x116bea => combat_body_block_195(e, &mut r, frame, pc)?,
                0x116c12 => combat_body_block_196(e, &mut r, frame, pc)?,
                0x116c18 => combat_body_block_197(e, &mut r, frame, pc)?,
                0x116c2e => combat_body_block_198(e, &mut r, frame, pc)?,
                0x116c32 => combat_body_block_199(e, &mut r, frame, pc)?,
                0x116c4e => combat_body_block_19a(e, &mut r, frame, pc)?,
                0x116c56 => combat_body_block_19b(e, &mut r, frame, pc)?,
                0x116c60 => combat_body_block_19c(e, &mut r, frame, pc)?,
                0x116c76 => combat_body_block_19d(e, &mut r, frame, pc)?,
                0x116c90 => combat_body_block_19e(e, &mut r, frame, pc)?,
                0x116c9a => combat_body_block_19f(e, &mut r, frame, pc)?,
                0x116ca4 => combat_body_block_1a0(e, &mut r, frame, pc)?,
                0x116cae => combat_body_block_1a1(e, &mut r, frame, pc)?,
                0x116cb8 => combat_body_block_1a2(e, &mut r, frame, pc)?,
                0x116cc2 => combat_body_block_1a3(e, &mut r, frame, pc)?,
                0x116ccc => combat_body_block_1a4(e, &mut r, frame, pc)?,
                0x116cd6 => combat_body_block_1a5(e, &mut r, frame, pc)?,
                0x116cde => combat_body_block_1a6(e, &mut r, frame, pc)?,
                0x116cf0 => combat_body_block_1a7(e, &mut r, frame, pc)?,
                0x116cf8 => combat_body_block_1a8(e, &mut r, frame, pc)?,
                0x116d00 => combat_body_block_1a9(e, &mut r, frame, pc)?,
                0x116d58 => combat_body_block_1aa(e, &mut r, frame, pc)?,
                0x116d62 => combat_body_block_1ab(e, &mut r, frame, pc)?,
                0x116d6a => combat_body_block_1ac(e, &mut r, frame, pc)?,
                0x116d84 => combat_body_block_1ad(e, &mut r, frame, pc)?,
                0x116db8 => combat_body_block_1ae(e, &mut r, frame, pc)?,
                0x116e18 => combat_body_block_1af(e, &mut r, frame, pc)?,
                0x116e22 => combat_body_block_1b0(e, &mut r, frame, pc)?,
                0x116e7c => combat_body_block_1b1(e, &mut r, frame, pc)?,
                0x116e84 => combat_body_block_1b2(e, &mut r, frame, pc)?,
                0x116eb6 => combat_body_block_1b3(e, &mut r, frame, pc)?,
                0x116ebe => combat_body_block_1b4(e, &mut r, frame, pc)?,
                0x116ef0 => combat_body_block_1b5(e, &mut r, frame, pc)?,
                0x116ef8 => combat_body_block_1b6(e, &mut r, frame, pc)?,
                0x116f52 => combat_body_block_1b7(e, &mut r, frame, pc)?,
                0x116f5a => combat_body_block_1b8(e, &mut r, frame, pc)?,
                0x116fb4 => combat_body_block_1b9(e, &mut r, frame, pc)?,
                0x116fbc => combat_body_block_1ba(e, &mut r, frame, pc)?,
                0x116fee => combat_body_block_1bb(e, &mut r, frame, pc)?,
                0x116ff6 => combat_body_block_1bc(e, &mut r, frame, pc)?,
                0x117028 => combat_body_block_1bd(e, &mut r, frame, pc)?,
                0x117030 => combat_body_block_1be(e, &mut r, frame, pc)?,
                0x11708a => combat_body_block_1bf(e, &mut r, frame, pc)?,
                0x117092 => combat_body_block_1c0(e, &mut r, frame, pc)?,
                0x1170c4 => combat_body_block_1c1(e, &mut r, frame, pc)?,
                0x1170cc => combat_body_block_1c2(e, &mut r, frame, pc)?,
                0x1170fe => combat_body_block_1c3(e, &mut r, frame, pc)?,
                0x117106 => combat_body_block_1c4(e, &mut r, frame, pc)?,
                0x117160 => combat_body_block_1c5(e, &mut r, frame, pc)?,
                0x117168 => combat_body_block_1c6(e, &mut r, frame, pc)?,
                0x1171c2 => combat_body_block_1c7(e, &mut r, frame, pc)?,
                0x1171ca => combat_body_block_1c8(e, &mut r, frame, pc)?,
                0x1171d4 => combat_body_block_1c9(e, &mut r, frame, pc)?,
                0x1171ee => combat_body_block_1ca(e, &mut r, frame, pc)?,
                0x1171f4 => combat_body_block_1cb(e, &mut r, frame, pc)?,
                0x117228 => combat_body_block_1cc(e, &mut r, frame, pc)?,
                0x117240 => combat_body_block_1cd(e, &mut r, frame, pc)?,
                0x11724a => combat_body_block_1ce(e, &mut r, frame, pc)?,
                0x117262 => combat_body_block_1cf(e, &mut r, frame, pc)?,
                0x11727a => combat_body_block_1d0(e, &mut r, frame, pc)?,
                0x117282 => combat_body_block_1d1(e, &mut r, frame, pc)?,
                0x11728e => combat_body_block_1d2(e, &mut r, frame, pc)?,
                0x1172a0 => combat_body_block_1d3(e, &mut r, frame, pc)?,
                0x1172aa => combat_body_block_1d4(e, &mut r, frame, pc)?,
                0x1172ba => combat_body_block_1d5(e, &mut r, frame, pc)?,
                0x1172c8 => combat_body_block_1d6(e, &mut r, frame, pc)?,
                0x1172d0 => combat_body_block_1d7(e, &mut r, frame, pc)?,
                0x117330 => combat_body_block_1d8(e, &mut r, frame, pc)?,
                0x117338 => combat_body_block_1d9(e, &mut r, frame, pc)?,
                0x117342 => combat_body_block_1da(e, &mut r, frame, pc)?,
                0x117344 => combat_body_block_1db(e, &mut r, frame, pc)?,
                _ => combat_body_block_1dc(e, &mut r, frame, pc)?,
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
crate::recovered_application_control::entry!(combat, 0x114378);
mod combat_body_block_0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114378: link.w a6, #$ffdc
        e.step()?;
        r.save_register(e, 14)?;
        r.put_address(6, r.address(7)?);
        r.offset_address(7, -36)?;
        // 0011437c: movem.l d3-d7/a2, -(a7)
        e.step()?;
        r.save_register(e, 10)?;
        r.save_register(e, 7)?;
        r.save_register(e, 6)?;
        r.save_register(e, 5)?;
        r.save_register(e, 4)?;
        r.save_register(e, 3)?;
        // 00114380: suba.l a2, a2
        e.step()?;
        r.put_address(2, 0);
        // 00114382: clr.w -$22(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xffffffde_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114386: clr.b -$b(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff5_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 0011438a: moveq #$0, d5
        e.step()?;
        r.put_data(5, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0011438c: clr.b -$5(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00114390: move.l $43872a.l, -(a7)
        e.step()?;
        let value_address = 0x43872a_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114396: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114398: MacToolbox 0xa93a
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa93a_u32 as u16))?;
        // 0011439a: move.l $43871e.l, -(a7)
        e.step()?;
        let value_address = 0x43871e_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001143a0: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001143a2: MacToolbox 0xa93a
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa93a_u32 as u16))?;
        // 001143a4: MacToolbox 0xa937
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa937_u32 as u16))?;
        // 001143a6: bsr.l $13e13c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1143ac_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13e13c_u32)?;
        // 001143ac: clr.w $447198.l
        e.step()?;
        let value = 0;
        let destination_address = 0x447198_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001143b2: clr.w $4427a6.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4427a6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001143b8: tst.w $43874e.l
        e.step()?;
        let value_address = 0x43874e_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001143be: bge.b $1143c6
        e.step()?;
        pc = if r.greater_equal()? {
            0x1143c6_u32
        } else {
            0x1143c0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_0_unit::combat_body_block_0;

mod combat_body_block_1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001143c0: move.w #$ffff, $8(a6)
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = r.address(6)?.wrapping_add(0x8_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1143c6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1_unit::combat_body_block_1;

mod combat_body_block_2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001143c6: pea.l $403eb2.l
        e.step()?;
        let value = 0x403eb2_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001143cc: bsr.l $136cfa
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1143d2_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x136cfa_u32)?;
        // 001143d2: pea.l $403eae.l
        e.step()?;
        let value = 0x403eae_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001143d8: pea.l $4389a4.l
        e.step()?;
        let value = 0x4389a4_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001143de: jsr $400070.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1143e4_u32)?;
        imported(e, r, NativeCall::Library(0x400070_u32))?;
        // 001143e4: movea.l a0, a2
        e.step()?;
        let value = r.address(0)?;
        r.put_address(2, value);
        // 001143e6: move.l a0, d0
        e.step()?;
        let value = r.address(0)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001143e8: lea.l $c(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xc_u32);
        r.put_address(7, value);
        // 001143ec: bne.b $1143fa
        e.step()?;
        pc = if r.not_equal()? {
            0x1143fa_u32
        } else {
            0x1143ee_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_2_unit::combat_body_block_2;

mod combat_body_block_3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001143ee: move.w #$7, -(a7)
        e.step()?;
        let value = 0x7_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001143f2: bsr.l $138be8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1143f8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x138be8_u32)?;
        // 001143f8: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1143fa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_3_unit::combat_body_block_3;

mod combat_body_block_4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001143fa: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001143fc: move.w $43874e.l, -(a7)
        e.step()?;
        let value_address = 0x43874e_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114402: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114408_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00114408: mulu.w #$55c, d0
        e.step()?;
        let source = 0x55c_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011440c: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0011440e: move.l d0, -(a7)
        e.step()?;
        let value = r.data(0, 32)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114410: move.l a2, -(a7)
        e.step()?;
        let value = r.address(2)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114412: jsr $400078.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114418_u32)?;
        imported(e, r, NativeCall::Library(0x400078_u32))?;
        // 00114418: move.l a2, -(a7)
        e.step()?;
        let value = r.address(2)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011441a: pea.l $1.w
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0011441e: pea.l $55c.w
        e.step()?;
        let value = 0x55c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00114422: pea.l $43979c.l
        e.step()?;
        let value = 0x43979c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00114428: jsr $400048.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11442e_u32)?;
        imported(e, r, NativeCall::Library(0x400048_u32))?;
        // 0011442e: move.l a2, -(a7)
        e.step()?;
        let value = r.address(2)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114430: jsr $400068.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114436_u32)?;
        imported(e, r, NativeCall::Library(0x400068_u32))?;
        // 00114436: tst.w $439ce4.l
        e.step()?;
        let value_address = 0x439ce4_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0011443c: lea.l $1e(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0x1e_u32);
        r.put_address(7, value);
        // 00114440: beq.b $11445a
        e.step()?;
        pc = if r.equal()? {
            0x11445a_u32
        } else {
            0x114442_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_4_unit::combat_body_block_4;

mod combat_body_block_5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114442: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114444: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114446: move.w $439ce4.l, -(a7)
        e.step()?;
        let value_address = 0x439ce4_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011444c: move.w #$ffff, -(a7)
        e.step()?;
        let value = 0xffff_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114450: bsr.l $15dc34
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114456_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x15dc34_u32)?;
        // 00114456: lea.l $a(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xa_u32);
        r.put_address(7, value);
        pc = 0x11445a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_5_unit::combat_body_block_5;

mod combat_body_block_6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011445a: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011445c: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011445e: move.l #$30035, -(a7)
        e.step()?;
        let value = 0x30035_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114464: bsr.l $15dc34
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11446a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x15dc34_u32)?;
        // 0011446a: move.b #$1, $44255c.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x44255c_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00114472: moveq #$1, d0
        e.step()?;
        r.put_data(0, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        // 00114474: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114476: move.w d0, $438776.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x438776_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011447c: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0011447e: lea.l $a(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xa_u32);
        r.put_address(7, value);
        // 00114482: bra.b $1144aa
        e.step()?;
        pc = 0x1144aa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_6_unit::combat_body_block_6;

mod combat_body_block_7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114484: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114486: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011448a: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00114490: cmpi.w #$1, $50(a0, d0.l)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x50_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00114496: bge.b $1144a8
        e.step()?;
        pc = if r.greater_equal()? {
            0x1144a8_u32
        } else {
            0x114498_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_7_unit::combat_body_block_7;

mod combat_body_block_8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114498: move.l #$10000, -(a7)
        e.step()?;
        let value = 0x10000_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011449e: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001144a0: bsr.l $161b80
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1144a6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x161b80_u32)?;
        // 001144a6: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1144a8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_8_unit::combat_body_block_8;

mod combat_body_block_9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001144a8: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x1144aa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_9_unit::combat_body_block_9;
mod combat_body_block_a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001144aa: move.b $44273a.l, d0
        e.step()?;
        let value_address = 0x44273a_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001144b0: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001144b2: cmp.w d0, d3
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001144b4: ble.b $114484
        e.step()?;
        pc = if r.less_equal()? {
            0x114484_u32
        } else {
            0x1144b6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_a_unit::combat_body_block_a;

mod combat_body_block_b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001144b6: move.w $8(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0x8_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001144ba: addq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001144bc: beq.b $1144e0
        e.step()?;
        pc = if r.equal()? {
            0x1144e0_u32
        } else {
            0x1144be_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_b_unit::combat_body_block_b;

mod combat_body_block_c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001144be: subq.w #$2, d0
        e.step()?;
        let source = 0x2_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001144c0: beq.b $1144c4
        e.step()?;
        pc = if r.equal()? {
            0x1144c4_u32
        } else {
            0x1144c2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_c_unit::combat_body_block_c;

mod combat_body_block_d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001144c2: bra.b $1144fa
        e.step()?;
        pc = 0x1144fa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_d_unit::combat_body_block_d;

mod combat_body_block_e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001144c4: pea.l $1771.w
        e.step()?;
        let value = 0x1771_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001144c8: move.l #$1e0064, -(a7)
        e.step()?;
        let value = 0x1e0064_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001144ce: pea.l $403e64.l
        e.step()?;
        let value = 0x403e64_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001144d4: bsr.l $123d1c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1144da_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x123d1c_u32)?;
        // 001144da: lea.l $c(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xc_u32);
        r.put_address(7, value);
        // 001144de: bra.b $1144fa
        e.step()?;
        pc = 0x1144fa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_e_unit::combat_body_block_e;

mod combat_body_block_f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001144e0: pea.l $1771.w
        e.step()?;
        let value = 0x1771_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001144e4: move.l #$1e0064, -(a7)
        e.step()?;
        let value = 0x1e0064_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001144ea: pea.l $403e16.l
        e.step()?;
        let value = 0x403e16_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001144f0: bsr.l $123d1c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1144f6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x123d1c_u32)?;
        // 001144f6: lea.l $c(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xc_u32);
        r.put_address(7, value);
        pc = 0x1144fa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_f_unit::combat_body_block_f;

mod combat_body_block_10_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_10(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001144fa: move.l $442774.l, -(a7)
        e.step()?;
        let value_address = 0x442774_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114500: bsr.l $151e24
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114506_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x151e24_u32)?;
        // 00114506: move.w #$37aa, -(a7)
        e.step()?;
        let value = 0x37aa_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011450a: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114510_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00114510: move.l #$10000, -(a7)
        e.step()?;
        let value = 0x10000_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114516: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114518: bsr.l $160a8e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11451e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x160a8e_u32)?;
        // 0011451e: bsr.l $11ce8e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114524_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x11ce8e_u32)?;
        // 00114524: move.w $a(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xa_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114528: move.w $8(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0x8_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011452c: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011452e: bsr.l $11d406
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114534_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x11d406_u32)?;
        // 00114534: move.l #$3d004b, -(a7)
        e.step()?;
        let value = 0x3d004b_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011453a: bsr.l $12382a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114540_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x12382a_u32)?;
        // 00114540: lea.l $18(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0x18_u32);
        r.put_address(7, value);
        // 00114544: bra.w $117338
        e.step()?;
        pc = 0x117338_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_10_unit::combat_body_block_10;

mod combat_body_block_11_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_11(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114548: move.w $44374c.l, d0
        e.step()?;
        let value_address = 0x44374c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011454e: cmp.w $4435c4.l, d0
        e.step()?;
        let source_address = 0x4435c4_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114554: blt.b $114570
        e.step()?;
        pc = if r.less()? {
            0x114570_u32
        } else {
            0x114556_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_11_unit::combat_body_block_11;

mod combat_body_block_12_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_12(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114556: clr.w $438776.l
        e.step()?;
        let value = 0;
        let destination_address = 0x438776_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011455c: move.l $442774.l, -(a7)
        e.step()?;
        let value_address = 0x442774_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114562: bsr.l $13107a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114568_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13107a_u32)?;
        // 00114568: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0011456a: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0011456c: bra.w $117344
        e.step()?;
        pc = 0x117344_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_12_unit::combat_body_block_12;

mod combat_body_block_13_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_13(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114570: pea.l $4471ee.l
        e.step()?;
        let value = 0x4471ee_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00114576: MacToolbox 0xa972
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa972_u32 as u16))?;
        // 00114578: clr.w $443316.l
        e.step()?;
        let value = 0;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011457e: tst.b $4427b2.l
        e.step()?;
        let value_address = 0x4427b2_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00114584: bne.b $1145be
        e.step()?;
        pc = if r.not_equal()? {
            0x1145be_u32
        } else {
            0x114586_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_13_unit::combat_body_block_13;

mod combat_body_block_14_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_14(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114586: tst.b $43565f.l
        e.step()?;
        let value_address = 0x43565f_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0011458c: bne.b $1145be
        e.step()?;
        pc = if r.not_equal()? {
            0x1145be_u32
        } else {
            0x11458e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_14_unit::combat_body_block_14;

mod combat_body_block_15_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_15(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011458e: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00114590: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114596: pea.l $4471b4.l
        e.step()?;
        let value = 0x4471b4_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0011459c: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 0011459e: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001145a0: bne.b $1145b4
        e.step()?;
        pc = if r.not_equal()? {
            0x1145b4_u32
        } else {
            0x1145a2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_15_unit::combat_body_block_15;

mod combat_body_block_16_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_16(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001145a2: move.l $43952c.l, -(a7)
        e.step()?;
        let value_address = 0x43952c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001145a8: MacToolbox 0xaa1c
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xaa1c_u32 as u16))?;
        // 001145aa: move.w #$14, $4471a4.l
        e.step()?;
        let value = 0x14_u32;
        let destination_address = 0x4471a4_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001145b2: bra.b $1145be
        e.step()?;
        pc = 0x1145be_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_16_unit::combat_body_block_16;

mod combat_body_block_17_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_17(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001145b4: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001145b6: bsr.l $15f210
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1145bc_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x15f210_u32)?;
        // 001145bc: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1145be_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_17_unit::combat_body_block_17;

mod combat_body_block_18_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_18(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001145be: tst.w $438778.l
        e.step()?;
        let value_address = 0x438778_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001145c4: beq.w $11578a
        e.step()?;
        pc = if r.equal()? {
            0x11578a_u32
        } else {
            0x1145c8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_18_unit::combat_body_block_18;

mod combat_body_block_19_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_19(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001145c8: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001145ce: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001145d2: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001145d8: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001145da: cmpi.b #$1, $109(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x109_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 001145e0: ble.w $115732
        e.step()?;
        pc = if r.less_equal()? {
            0x115732_u32
        } else {
            0x1145e4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_19_unit::combat_body_block_19;
mod combat_body_block_1a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001145e4: MacToolbox 0xa9b4
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9b4_u32 as u16))?;
        // 001145e6: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001145e8: bsr.l $133d46
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1145ee_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133d46_u32)?;
        // 001145ee: MacToolbox 0xa974
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa974_u32 as u16))?;
        // 001145f0: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001145f2: beq.b $11465e
        e.step()?;
        pc = if r.equal()? {
            0x11465e_u32
        } else {
            0x1145f4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1a_unit::combat_body_block_1a;

mod combat_body_block_1b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001145f4: pea.l $447214.l
        e.step()?;
        let value = 0x447214_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001145fa: MacToolbox 0xa972
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa972_u32 as u16))?;
        // 001145fc: pea.l $447214.l
        e.step()?;
        let value = 0x447214_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00114602: MacToolbox 0xa870
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa870_u32 as u16))?;
        // 00114604: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00114606: move.l $447214.l, -(a7)
        e.step()?;
        let value_address = 0x447214_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011460c: pea.l -$1c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00114610: MacToolbox 0xa92c
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa92c_u32 as u16))?;
        // 00114612: move.w (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114614: move.w d0, $4471a6.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4471a6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011461a: move.w $4471a6.l, d0
        e.step()?;
        let value_address = 0x4471a6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114620: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114622: beq.b $114626
        e.step()?;
        pc = if r.equal()? {
            0x114626_u32
        } else {
            0x114624_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1b_unit::combat_body_block_1b;

mod combat_body_block_1c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114624: bra.b $11465e
        e.step()?;
        pc = 0x11465e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1c_unit::combat_body_block_1c;

mod combat_body_block_1d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114626: move.w $4471a6.l, d0
        e.step()?;
        let value_address = 0x4471a6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011462c: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011462e: beq.b $114632
        e.step()?;
        pc = if r.equal()? {
            0x114632_u32
        } else {
            0x114630_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1d_unit::combat_body_block_1d;

mod combat_body_block_1e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114630: bra.b $11465e
        e.step()?;
        pc = 0x11465e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1e_unit::combat_body_block_1e;

mod combat_body_block_1f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114632: bsr.l $13e13c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114638_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13e13c_u32)?;
        // 00114638: subq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 0011463a: move.l $447214.l, -(a7)
        e.step()?;
        let value_address = 0x447214_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00114640: MacToolbox 0xa93d
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa93d_u32 as u16))?;
        // 00114642: move.l (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = e.l(value_address)?;
        r.offset_address(7, 4)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114644: move.l d0, $438742.l
        e.step()?;
        let value = r.data(0, 32)?;
        let destination_address = 0x438742_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011464a: bsr.l $1279bc
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114650_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1279bc_u32)?;
        // 00114650: tst.b $4386b2.l
        e.step()?;
        let value_address = 0x4386b2_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00114656: beq.b $11465e
        e.step()?;
        pc = if r.equal()? {
            0x11465e_u32
        } else {
            0x114658_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1f_unit::combat_body_block_1f;

mod combat_body_block_20_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_20(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114658: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0011465a: bra.w $117344
        e.step()?;
        pc = 0x117344_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_20_unit::combat_body_block_20;

mod combat_body_block_21_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_21(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011465e: clr.b -$6(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00114662: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114668: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011466c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114672: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00114674: tst.b $109(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x109_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00114678: ble.b $114694
        e.step()?;
        pc = if r.less_equal()? {
            0x114694_u32
        } else {
            0x11467a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_21_unit::combat_body_block_21;

mod combat_body_block_22_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_22(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011467a: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114680: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114684: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0011468a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0011468c: move.b #$1, $104(a0)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x104_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00114692: bra.b $1146aa
        e.step()?;
        pc = 0x1146aa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_22_unit::combat_body_block_22;

mod combat_body_block_23_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_23(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114694: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011469a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011469e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001146a4: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001146a6: clr.b $104(a0)
        e.step()?;
        let value = 0;
        let destination_address = r.address(0)?.wrapping_add(0x104_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x1146aa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_23_unit::combat_body_block_23;

mod combat_body_block_24_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_24(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001146aa: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001146b0: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001146b4: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001146ba: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001146bc: tst.b $b2(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xb2_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001146c0: bne.w $115140
        e.step()?;
        pc = if r.not_equal()? {
            0x115140_u32
        } else {
            0x1146c4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_24_unit::combat_body_block_24;

mod combat_body_block_25_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_25(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001146c4: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001146ca: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001146ce: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001146d4: move.w $48(a0, d0.l), -$24(a6)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x48_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xffffffdc_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001146da: move.w #$64, -(a7)
        e.step()?;
        let value = 0x64_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001146de: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1146e4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 001146e4: cmp.w -$24(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffffdc_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001146e8: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001146ea: bgt.w $114794
        e.step()?;
        pc = if r.greater()? {
            0x114794_u32
        } else {
            0x1146ee_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_25_unit::combat_body_block_25;

mod combat_body_block_26_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_26(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001146ee: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001146f4: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001146f8: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001146fe: move.w $447196.l, d1
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00114704: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00114708: lea.l $439d3c.l, a1
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(1, value);
        // 0011470e: move.w $26(a0, d0.l), $4e(a1, d1.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x26_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r
            .address(1)?
            .wrapping_add(0x4e_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114714: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011471a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011471e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114724: move.w $4e(a0, d0.l), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x4e_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114728: bsr.l $130d38
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11472e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x130d38_u32)?;
        // 0011472e: tst.w $45365c.l
        e.step()?;
        let value_address = 0x45365c_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00114734: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00114736: beq.b $114756
        e.step()?;
        pc = if r.equal()? {
            0x114756_u32
        } else {
            0x114738_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_26_unit::combat_body_block_26;

mod combat_body_block_27_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_27(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114738: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011473e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114742: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114748: move.w $4c(a0, d0.l), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x4c_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011474c: bsr.l $130e54
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114752_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x130e54_u32)?;
        // 00114752: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00114754: bra.b $114760
        e.step()?;
        pc = 0x114760_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_27_unit::combat_body_block_27;

mod combat_body_block_28_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_28(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114756: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114758: bsr.l $130e54
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11475e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x130e54_u32)?;
        // 0011475e: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x114760_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_28_unit::combat_body_block_28;

mod combat_body_block_29_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_29(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114760: cmpi.w #$44c, $453794.l
        e.step()?;
        let source = 0x44c_u32;
        let destination_address = 0x453794_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00114768: ble.b $114784
        e.step()?;
        pc = if r.less_equal()? {
            0x114784_u32
        } else {
            0x11476a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_29_unit::combat_body_block_29;
mod combat_body_block_2a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_2a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011476a: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011476c: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 00114772: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114778: move.w (a0, d0.w * 2), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011477c: bsr.l $14c6c2
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114782_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x14c6c2_u32)?;
        // 00114782: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x114784_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_2a_unit::combat_body_block_2a;

mod combat_body_block_2b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_2b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114784: move.w #$33, -(a7)
        e.step()?;
        let value = 0x33_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114788: bsr.l $13154c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11478e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13154c_u32)?;
        // 0011478e: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00114790: bra.w $114b78
        e.step()?;
        pc = 0x114b78_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_2b_unit::combat_body_block_2b;

mod combat_body_block_2c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_2c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114794: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011479a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011479e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001147a4: move.w $42(a0, d0.l), -$24(a6)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x42_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xffffffdc_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001147aa: move.w #$64, -(a7)
        e.step()?;
        let value = 0x64_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001147ae: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1147b4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 001147b4: cmp.w -$24(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffffdc_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001147b8: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001147ba: bgt.w $11511e
        e.step()?;
        pc = if r.greater()? {
            0x11511e_u32
        } else {
            0x1147be_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_2c_unit::combat_body_block_2c;

mod combat_body_block_2d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_2d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001147be: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001147c4: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001147c8: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001147ce: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001147d0: tst.b $b7(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xb7_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001147d4: bne.w $115140
        e.step()?;
        pc = if r.not_equal()? {
            0x115140_u32
        } else {
            0x1147d8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_2d_unit::combat_body_block_2d;

mod combat_body_block_2e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_2e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001147d8: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001147de: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001147e2: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001147e8: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001147ea: tst.b $cf(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xcf_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001147ee: bne.w $115140
        e.step()?;
        pc = if r.not_equal()? {
            0x115140_u32
        } else {
            0x1147f2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_2e_unit::combat_body_block_2e;

mod combat_body_block_2f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_2f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001147f2: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001147f8: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001147fc: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114802: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00114804: tst.b $d9(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xd9_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00114808: bne.w $115140
        e.step()?;
        pc = if r.not_equal()? {
            0x115140_u32
        } else {
            0x11480c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_2f_unit::combat_body_block_2f;

mod combat_body_block_30_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_30(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011480c: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114812: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114816: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0011481c: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0011481e: tst.b $b3(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xb3_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00114822: bne.w $115140
        e.step()?;
        pc = if r.not_equal()? {
            0x115140_u32
        } else {
            0x114826_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_30_unit::combat_body_block_30;

mod combat_body_block_31_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_31(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114826: tst.b $44274b.l
        e.step()?;
        let value_address = 0x44274b_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0011482c: bne.w $115140
        e.step()?;
        pc = if r.not_equal()? {
            0x115140_u32
        } else {
            0x114830_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_31_unit::combat_body_block_31;

mod combat_body_block_32_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_32(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114830: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114836: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011483a: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114840: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00114842: tst.b $105(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x105_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00114846: bne.w $115140
        e.step()?;
        pc = if r.not_equal()? {
            0x115140_u32
        } else {
            0x11484a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_32_unit::combat_body_block_32;

mod combat_body_block_33_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_33(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011484a: clr.b -$c(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 0011484e: bra.w $1150ee
        e.step()?;
        pc = 0x1150ee_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_33_unit::combat_body_block_33;

mod combat_body_block_34_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_34(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114852: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00114854: bra.b $1148ae
        e.step()?;
        pc = 0x1148ae_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_34_unit::combat_body_block_34;

mod combat_body_block_35_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_35(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114856: pea.l $9.w
        e.step()?;
        let value = 0x9_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0011485a: bsr.l $13849e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114860_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13849e_u32)?;
        // 00114860: move.b d0, -$1(a6)
        e.step()?;
        let value = r.data(0, 8)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffff_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00114864: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011486a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011486e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114874: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00114876: move.b -$1(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffff_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0011487a: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011487c: cmpi.w #$44c, $12(a0, d0.l * 2)
        e.step()?;
        let source = 0x44c_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x12_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00114882: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00114884: ble.b $1148ac
        e.step()?;
        pc = if r.less_equal()? {
            0x1148ac_u32
        } else {
            0x114886_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_35_unit::combat_body_block_35;

mod combat_body_block_36_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_36(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114886: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011488c: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114890: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114896: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00114898: move.b -$1(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffff_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0011489c: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011489e: move.w $12(a0, d0.l * 2), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x12_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001148a2: bsr.l $13145c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1148a8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13145c_u32)?;
        // 001148a8: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001148aa: bra.b $1148b8
        e.step()?;
        pc = 0x1148b8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_36_unit::combat_body_block_36;

mod combat_body_block_37_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_37(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001148ac: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x1148ae_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_37_unit::combat_body_block_37;

mod combat_body_block_38_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_38(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001148ae: cmpi.w #$1e, d3
        e.step()?;
        let source = 0x1e_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001148b2: blt.b $114856
        e.step()?;
        pc = if r.less()? {
            0x114856_u32
        } else {
            0x1148b4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_38_unit::combat_body_block_38;

mod combat_body_block_39_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_39(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001148b4: bra.w $11510a
        e.step()?;
        pc = 0x11510a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_39_unit::combat_body_block_39;
mod combat_body_block_3a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_3a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001148b8: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001148ba: bra.b $1148c8
        e.step()?;
        pc = 0x1148c8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_3a_unit::combat_body_block_3a;

mod combat_body_block_3b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_3b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001148bc: lea.l $4434e0.l, a0
        e.step()?;
        let value = 0x4434e0_u32;
        r.put_address(0, value);
        // 001148c2: clr.w (a0, d3.w * 2)
        e.step()?;
        let value = 0;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001148c6: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x1148c8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_3b_unit::combat_body_block_3b;

mod combat_body_block_3c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_3c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001148c8: cmpi.w #$6e, d3
        e.step()?;
        let source = 0x6e_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001148cc: blt.b $1148bc
        e.step()?;
        pc = if r.less()? {
            0x1148bc_u32
        } else {
            0x1148ce_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_3c_unit::combat_body_block_3c;

mod combat_body_block_3d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_3d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001148ce: cmpi.b #$c, $44374f.l
        e.step()?;
        let source = 0xc_u32;
        let destination_address = 0x44374f_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 001148d6: bne.w $114970
        e.step()?;
        pc = if r.not_equal()? {
            0x114970_u32
        } else {
            0x1148da_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_3d_unit::combat_body_block_3d;

mod combat_body_block_3e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_3e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001148da: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001148dc: bra.b $114900
        e.step()?;
        pc = 0x114900_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_3e_unit::combat_body_block_3e;

mod combat_body_block_3f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_3f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001148de: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001148e0: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001148e4: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001148ea: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001148ec: tst.b $9df(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x9df_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001148f0: beq.b $1148fe
        e.step()?;
        pc = if r.equal()? {
            0x1148fe_u32
        } else {
            0x1148f2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_3f_unit::combat_body_block_3f;

mod combat_body_block_40_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_40(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001148f2: lea.l $4434e0.l, a0
        e.step()?;
        let value = 0x4434e0_u32;
        r.put_address(0, value);
        // 001148f8: move.w #$1, (a0, d3.w * 2)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1148fe_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_40_unit::combat_body_block_40;

mod combat_body_block_41_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_41(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001148fe: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x114900_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_41_unit::combat_body_block_41;

mod combat_body_block_42_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_42(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114900: move.b $44273a.l, d0
        e.step()?;
        let value_address = 0x44273a_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114906: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114908: cmp.w d0, d3
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0011490a: ble.b $1148de
        e.step()?;
        pc = if r.less_equal()? {
            0x1148de_u32
        } else {
            0x11490c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_42_unit::combat_body_block_42;

mod combat_body_block_43_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_43(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011490c: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0011490e: bra.b $114936
        e.step()?;
        pc = 0x114936_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_43_unit::combat_body_block_43;

mod combat_body_block_44_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_44(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114910: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114912: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114916: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0011491c: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0011491e: tst.w $110(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x110_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00114922: ble.b $114934
        e.step()?;
        pc = if r.less_equal()? {
            0x114934_u32
        } else {
            0x114924_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_44_unit::combat_body_block_44;

mod combat_body_block_45_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_45(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114924: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 00114926: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114928: lea.l $4434e0.l, a0
        e.step()?;
        let value = 0x4434e0_u32;
        r.put_address(0, value);
        // 0011492e: move.w #$1, (a0, d0.w * 2)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x114934_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_45_unit::combat_body_block_45;

mod combat_body_block_46_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_46(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114934: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x114936_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_46_unit::combat_body_block_46;

mod combat_body_block_47_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_47(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114936: cmp.w $43874a.l, d3
        e.step()?;
        let source_address = 0x43874a_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0011493c: blt.b $114910
        e.step()?;
        pc = if r.less()? {
            0x114910_u32
        } else {
            0x11493e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_47_unit::combat_body_block_47;

mod combat_body_block_48_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_48(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011493e: move.b #$1, -$6(a6)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00114944: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00114946: moveq #$1, d0
        e.step()?;
        r.put_data(0, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        // 00114948: move.w d0, $438760.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x438760_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011494e: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 00114950: add.w $447196.l, d0
        e.step()?;
        let source_address = 0x447196_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114956: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114958: bsr.l $11f1a8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11495e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x11f1a8_u32)?;
        // 0011495e: bsr.l $159192
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114964_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x159192_u32)?;
        // 00114964: bsr.l $151022
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11496a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x151022_u32)?;
        // 0011496a: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0011496c: bra.w $1150d8
        e.step()?;
        pc = 0x1150d8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_48_unit::combat_body_block_48;

mod combat_body_block_49_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_49(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114970: cmpi.b #$9, $44374f.l
        e.step()?;
        let source = 0x9_u32;
        let destination_address = 0x44374f_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00114978: bne.w $114a74
        e.step()?;
        pc = if r.not_equal()? {
            0x114a74_u32
        } else {
            0x11497c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_49_unit::combat_body_block_49;
mod combat_body_block_4a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_4a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011497c: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0011497e: bra.b $1149c8
        e.step()?;
        pc = 0x1149c8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_4a_unit::combat_body_block_4a;

mod combat_body_block_4b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_4b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114980: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114986: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011498a: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114990: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00114992: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00114996: lea.l $448aa0.l, a1
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(1, value);
        // 0011499c: move.w $10(a1, d1.l), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001149a0: cmp.w $e(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001149a4: bne.b $1149c6
        e.step()?;
        pc = if r.not_equal()? {
            0x1149c6_u32
        } else {
            0x1149a6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_4b_unit::combat_body_block_4b;

mod combat_body_block_4c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_4c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001149a6: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001149a8: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001149ac: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001149b2: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001149b4: tst.b $9df(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x9df_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001149b8: beq.b $1149c6
        e.step()?;
        pc = if r.equal()? {
            0x1149c6_u32
        } else {
            0x1149ba_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_4c_unit::combat_body_block_4c;

mod combat_body_block_4d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_4d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001149ba: lea.l $4434e0.l, a0
        e.step()?;
        let value = 0x4434e0_u32;
        r.put_address(0, value);
        // 001149c0: move.w #$1, (a0, d3.w * 2)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1149c6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_4d_unit::combat_body_block_4d;

mod combat_body_block_4e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_4e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001149c6: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x1149c8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_4e_unit::combat_body_block_4e;

mod combat_body_block_4f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_4f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001149c8: move.b $44273a.l, d0
        e.step()?;
        let value_address = 0x44273a_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001149ce: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001149d0: cmp.w d0, d3
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001149d2: ble.b $114980
        e.step()?;
        pc = if r.less_equal()? {
            0x114980_u32
        } else {
            0x1149d4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_4f_unit::combat_body_block_4f;

mod combat_body_block_50_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_50(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001149d4: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001149d6: bra.b $114a24
        e.step()?;
        pc = 0x114a24_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_50_unit::combat_body_block_50;

mod combat_body_block_51_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_51(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001149d8: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001149de: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001149e2: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001149e8: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001149ea: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 001149ee: lea.l $439d3c.l, a1
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(1, value);
        // 001149f4: move.w $e(a1, d1.l), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001149f8: cmp.w $e(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001149fc: bne.b $114a22
        e.step()?;
        pc = if r.not_equal()? {
            0x114a22_u32
        } else {
            0x1149fe_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_51_unit::combat_body_block_51;

mod combat_body_block_52_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_52(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001149fe: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114a00: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114a04: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114a0a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00114a0c: tst.w $110(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x110_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00114a10: ble.b $114a22
        e.step()?;
        pc = if r.less_equal()? {
            0x114a22_u32
        } else {
            0x114a12_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_52_unit::combat_body_block_52;

mod combat_body_block_53_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_53(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114a12: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 00114a14: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114a16: lea.l $4434e0.l, a0
        e.step()?;
        let value = 0x4434e0_u32;
        r.put_address(0, value);
        // 00114a1c: move.w #$1, (a0, d0.w * 2)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x114a22_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_53_unit::combat_body_block_53;

mod combat_body_block_54_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_54(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114a22: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x114a24_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_54_unit::combat_body_block_54;

mod combat_body_block_55_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_55(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114a24: cmp.w $43874a.l, d3
        e.step()?;
        let source_address = 0x43874a_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114a2a: blt.b $1149d8
        e.step()?;
        pc = if r.less()? {
            0x1149d8_u32
        } else {
            0x114a2c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_55_unit::combat_body_block_55;

mod combat_body_block_56_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_56(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114a2c: move.b #$1, -$6(a6)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00114a32: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00114a34: moveq #$1, d0
        e.step()?;
        r.put_data(0, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        // 00114a36: move.w d0, $438760.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x438760_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114a3c: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 00114a3e: add.w $447196.l, d0
        e.step()?;
        let source_address = 0x447196_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114a44: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114a46: bsr.l $11f1a8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114a4c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x11f1a8_u32)?;
        // 00114a4c: bsr.l $159192
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114a52_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x159192_u32)?;
        // 00114a52: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114a58: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114a5c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114a62: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00114a64: clr.b $104(a0)
        e.step()?;
        let value = 0;
        let destination_address = r.address(0)?.wrapping_add(0x104_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00114a68: bsr.l $151022
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114a6e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x151022_u32)?;
        // 00114a6e: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00114a70: bra.w $1150d8
        e.step()?;
        pc = 0x1150d8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_56_unit::combat_body_block_56;

mod combat_body_block_57_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_57(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114a74: cmpi.b #$a, $44374f.l
        e.step()?;
        let source = 0xa_u32;
        let destination_address = 0x44374f_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00114a7c: bne.w $114b78
        e.step()?;
        pc = if r.not_equal()? {
            0x114b78_u32
        } else {
            0x114a80_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_57_unit::combat_body_block_57;

mod combat_body_block_58_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_58(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114a80: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00114a82: bra.b $114acc
        e.step()?;
        pc = 0x114acc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_58_unit::combat_body_block_58;

mod combat_body_block_59_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_59(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114a84: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114a8a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114a8e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114a94: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00114a96: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00114a9a: lea.l $448aa0.l, a1
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(1, value);
        // 00114aa0: move.w $10(a1, d1.l), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00114aa4: cmp.w $e(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114aa8: beq.b $114aca
        e.step()?;
        pc = if r.equal()? {
            0x114aca_u32
        } else {
            0x114aaa_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_59_unit::combat_body_block_59;
mod combat_body_block_5a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_5a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114aaa: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114aac: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114ab0: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00114ab6: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00114ab8: tst.b $9df(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x9df_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00114abc: beq.b $114aca
        e.step()?;
        pc = if r.equal()? {
            0x114aca_u32
        } else {
            0x114abe_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_5a_unit::combat_body_block_5a;

mod combat_body_block_5b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_5b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114abe: lea.l $4434e0.l, a0
        e.step()?;
        let value = 0x4434e0_u32;
        r.put_address(0, value);
        // 00114ac4: move.w #$1, (a0, d3.w * 2)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x114aca_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_5b_unit::combat_body_block_5b;

mod combat_body_block_5c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_5c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114aca: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x114acc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_5c_unit::combat_body_block_5c;

mod combat_body_block_5d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_5d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114acc: move.b $44273a.l, d0
        e.step()?;
        let value_address = 0x44273a_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114ad2: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114ad4: cmp.w d0, d3
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114ad6: ble.b $114a84
        e.step()?;
        pc = if r.less_equal()? {
            0x114a84_u32
        } else {
            0x114ad8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_5d_unit::combat_body_block_5d;

mod combat_body_block_5e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_5e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114ad8: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00114ada: bra.b $114b28
        e.step()?;
        pc = 0x114b28_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_5e_unit::combat_body_block_5e;

mod combat_body_block_5f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_5f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114adc: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114ae2: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114ae6: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114aec: move.w d3, d1
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00114aee: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00114af2: lea.l $439d3c.l, a1
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(1, value);
        // 00114af8: move.w $e(a1, d1.l), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00114afc: cmp.w $e(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114b00: beq.b $114b26
        e.step()?;
        pc = if r.equal()? {
            0x114b26_u32
        } else {
            0x114b02_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_5f_unit::combat_body_block_5f;

mod combat_body_block_60_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_60(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114b02: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114b04: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114b08: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114b0e: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00114b10: tst.w $110(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x110_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00114b14: ble.b $114b26
        e.step()?;
        pc = if r.less_equal()? {
            0x114b26_u32
        } else {
            0x114b16_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_60_unit::combat_body_block_60;

mod combat_body_block_61_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_61(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114b16: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 00114b18: add.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114b1a: lea.l $4434e0.l, a0
        e.step()?;
        let value = 0x4434e0_u32;
        r.put_address(0, value);
        // 00114b20: move.w #$1, (a0, d0.w * 2)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x114b26_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_61_unit::combat_body_block_61;

mod combat_body_block_62_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_62(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114b26: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x114b28_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_62_unit::combat_body_block_62;

mod combat_body_block_63_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_63(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114b28: cmp.w $43874a.l, d3
        e.step()?;
        let source_address = 0x43874a_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114b2e: blt.b $114adc
        e.step()?;
        pc = if r.less()? {
            0x114adc_u32
        } else {
            0x114b30_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_63_unit::combat_body_block_63;

mod combat_body_block_64_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_64(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114b30: move.b #$1, -$6(a6)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00114b36: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00114b38: moveq #$1, d0
        e.step()?;
        r.put_data(0, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        // 00114b3a: move.w d0, $438760.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x438760_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114b40: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 00114b42: add.w $447196.l, d0
        e.step()?;
        let source_address = 0x447196_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114b48: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114b4a: bsr.l $11f1a8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114b50_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x11f1a8_u32)?;
        // 00114b50: bsr.l $159192
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114b56_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x159192_u32)?;
        // 00114b56: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114b5c: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114b60: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114b66: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00114b68: clr.b $104(a0)
        e.step()?;
        let value = 0;
        let destination_address = r.address(0)?.wrapping_add(0x104_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00114b6c: bsr.l $151022
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114b72_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x151022_u32)?;
        // 00114b72: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00114b74: bra.w $1150d8
        e.step()?;
        pc = 0x1150d8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_64_unit::combat_body_block_64;

mod combat_body_block_65_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_65(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114b78: clr.b -$2(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00114b7c: moveq #$0, d7
        e.step()?;
        r.put_data(7, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00114b7e: bsr.l $13822a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114b84_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13822a_u32)?;
        // 00114b84: move.w #$7, -(a7)
        e.step()?;
        let value = 0x7_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114b88: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114b8e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00114b8e: move.w d0, $4435cc.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4435cc_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114b94: cmpi.b #$9, $443761.l
        e.step()?;
        let source = 0x9_u32;
        let destination_address = 0x443761_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00114b9c: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00114b9e: bne.b $114baa
        e.step()?;
        pc = if r.not_equal()? {
            0x114baa_u32
        } else {
            0x114ba0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_65_unit::combat_body_block_65;

mod combat_body_block_66_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_66(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114ba0: move.w $4537b4.l, $4435cc.l
        e.step()?;
        let value_address = 0x4537b4_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4435cc_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x114baa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_66_unit::combat_body_block_66;

mod combat_body_block_67_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_67(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114baa: move.b $443751.l, d0
        e.step()?;
        let value_address = 0x443751_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114bb0: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114bb2: muls.w $4435cc.l, d0
        e.step()?;
        let source_address = 0x4435cc_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114bb8: move.b $443750.l, d1
        e.step()?;
        let value_address = 0x443750_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(1, 8, value);
        r.move_flags(value, 8);
        // 00114bbe: ext.w d1
        e.step()?;
        let source = r.data(1, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00114bc0: add.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00114bc2: move.w d1, -(a7)
        e.step()?;
        let value = r.data(1, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114bc4: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114bca_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00114bca: move.w d0, $4435bc.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4435bc_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114bd0: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114bd6: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114bda: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114be0: tst.w $10(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00114be4: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00114be6: beq.b $114bee
        e.step()?;
        pc = if r.equal()? {
            0x114bee_u32
        } else {
            0x114be8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_67_unit::combat_body_block_67;

mod combat_body_block_68_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_68(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114be8: addq.w #$1, $4435bc.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4435bc_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x114bee_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_68_unit::combat_body_block_68;

mod combat_body_block_69_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_69(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114bee: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114bf4: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114bf8: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114bfe: move.w $10(a0, d0.l), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114c02: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114c08_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00114c08: subq.w #$3, d0
        e.step()?;
        let source = 0x3_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114c0a: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00114c0c: bne.b $114c14
        e.step()?;
        pc = if r.not_equal()? {
            0x114c14_u32
        } else {
            0x114c0e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_69_unit::combat_body_block_69;
mod combat_body_block_6a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_6a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114c0e: addq.w #$1, $4435bc.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4435bc_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x114c14_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_6a_unit::combat_body_block_6a;

mod combat_body_block_6b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_6b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114c14: move.b $443757.l, d0
        e.step()?;
        let value_address = 0x443757_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114c1a: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114c1c: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114c1e: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114c24: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114c28: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114c2e: move.w $e(a0, d0.l), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114c32: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 00114c34: add.w $447196.l, d0
        e.step()?;
        let source_address = 0x447196_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114c3a: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114c3c: bsr.l $125538
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114c42_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x125538_u32)?;
        // 00114c42: move.b d0, -$2(a6)
        e.step()?;
        let value = r.data(0, 8)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00114c46: tst.b -$2(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00114c4a: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00114c4c: beq.w $11510a
        e.step()?;
        pc = if r.equal()? {
            0x11510a_u32
        } else {
            0x114c50_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_6b_unit::combat_body_block_6b;

mod combat_body_block_6c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_6c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114c50: tst.b $44374f.l
        e.step()?;
        let value_address = 0x44374f_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00114c56: bne.b $114c6e
        e.step()?;
        pc = if r.not_equal()? {
            0x114c6e_u32
        } else {
            0x114c58_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_6c_unit::combat_body_block_6c;

mod combat_body_block_6d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_6d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114c58: move.b -$2(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114c5c: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114c5e: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114c60: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114c66_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00114c66: move.w d0, $4435cc.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4435cc_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114c6c: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x114c6e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_6d_unit::combat_body_block_6d;

mod combat_body_block_6e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_6e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114c6e: cmpi.w #$7, $4435cc.l
        e.step()?;
        let source = 0x7_u32;
        let destination_address = 0x4435cc_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00114c76: ble.b $114c80
        e.step()?;
        pc = if r.less_equal()? {
            0x114c80_u32
        } else {
            0x114c78_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_6e_unit::combat_body_block_6e;

mod combat_body_block_6f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_6f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114c78: move.w #$7, $4435cc.l
        e.step()?;
        let value = 0x7_u32;
        let destination_address = 0x4435cc_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x114c80_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_6f_unit::combat_body_block_6f;

mod combat_body_block_70_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_70(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114c80: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114c86: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114c8a: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114c90: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00114c92: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00114c94: move.b $44376c.l, d0
        e.step()?;
        let value_address = 0x44376c_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114c9a: muls.w $4435cc.l, d0
        e.step()?;
        let source_address = 0x4435cc_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114ca0: cmp.w $10e(a0), d0
        e.step()?;
        let source_address = r.address(0)?.wrapping_add(0x10e_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114ca4: ble.b $114cb8
        e.step()?;
        pc = if r.less_equal()? {
            0x114cb8_u32
        } else {
            0x114ca6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_70_unit::combat_body_block_70;

mod combat_body_block_71_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_71(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114ca6: subq.w #$1, $4435cc.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4435cc_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00114cac: tst.w $4435cc.l
        e.step()?;
        let value_address = 0x4435cc_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00114cb2: beq.w $11510a
        e.step()?;
        pc = if r.equal()? {
            0x11510a_u32
        } else {
            0x114cb6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_71_unit::combat_body_block_71;

mod combat_body_block_72_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_72(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114cb6: bra.b $114c80
        e.step()?;
        pc = 0x114c80_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_72_unit::combat_body_block_72;

mod combat_body_block_73_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_73(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114cb8: move.w $4435cc.l, $4435c0.l
        e.step()?;
        let value_address = 0x4435cc_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4435c0_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114cc2: move.b $4435c1.l, -$2(a6)
        e.step()?;
        let value_address = 0x4435c1_u32;
        let value = u32::from(e.b(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00114cca: tst.b $44374f.l
        e.step()?;
        let value_address = 0x44374f_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00114cd0: ble.b $114cea
        e.step()?;
        pc = if r.less_equal()? {
            0x114cea_u32
        } else {
            0x114cd2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_73_unit::combat_body_block_73;

mod combat_body_block_74_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_74(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114cd2: cmpi.b #$9, $443761.l
        e.step()?;
        let source = 0x9_u32;
        let destination_address = 0x443761_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00114cda: beq.b $114cea
        e.step()?;
        pc = if r.equal()? {
            0x114cea_u32
        } else {
            0x114cdc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_74_unit::combat_body_block_74;

mod combat_body_block_75_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_75(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114cdc: move.w #$1, $4435c0.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x4435c0_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114ce4: moveq #$1, d0
        e.step()?;
        r.put_data(0, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        // 00114ce6: move.b d0, -$2(a6)
        e.step()?;
        let value = r.data(0, 8)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x114cea_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_75_unit::combat_body_block_75;

mod combat_body_block_76_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_76(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114cea: cmpi.b #$3, $44374f.l
        e.step()?;
        let source = 0x3_u32;
        let destination_address = 0x44374f_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00114cf2: bne.w $114dd6
        e.step()?;
        pc = if r.not_equal()? {
            0x114dd6_u32
        } else {
            0x114cf6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_76_unit::combat_body_block_76;

mod combat_body_block_77_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_77(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114cf6: cmpi.b #$a, $44376d.l
        e.step()?;
        let source = 0xa_u32;
        let destination_address = 0x44376d_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00114cfe: bne.w $114dd6
        e.step()?;
        pc = if r.not_equal()? {
            0x114dd6_u32
        } else {
            0x114d02_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_77_unit::combat_body_block_77;

mod combat_body_block_78_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_78(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114d02: move.w $4436b4.l, d0
        e.step()?;
        let value_address = 0x4436b4_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114d08: cmp.w $4435bc.l, d0
        e.step()?;
        let source_address = 0x4435bc_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114d0e: bge.w $114dd6
        e.step()?;
        pc = if r.greater_equal()? {
            0x114dd6_u32
        } else {
            0x114d12_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_78_unit::combat_body_block_78;

mod combat_body_block_79_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_79(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114d12: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00114d14: bra.w $114dca
        e.step()?;
        pc = 0x114dca_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_79_unit::combat_body_block_79;
mod combat_body_block_7a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_7a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114d18: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 00114d1a: add.w $43874a.l, d0
        e.step()?;
        let source_address = 0x43874a_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114d20: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114d22: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114d28_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00114d28: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114d2a: move.w d0, d4
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(4, 16, value);
        r.move_flags(value, 16);
        // 00114d2c: lea.l $4435d6.l, a0
        e.step()?;
        let value = 0x4435d6_u32;
        r.put_address(0, value);
        // 00114d32: move.w (a0, d4.w * 2), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(4, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114d36: cmp.w $4435bc.l, d0
        e.step()?;
        let source_address = 0x4435bc_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114d3c: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00114d3e: bgt.w $114dc8
        e.step()?;
        pc = if r.greater()? {
            0x114dc8_u32
        } else {
            0x114d42_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_7a_unit::combat_body_block_7a;

mod combat_body_block_7b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_7b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114d42: cmpi.w #$9, d4
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114d46: bge.b $114d84
        e.step()?;
        pc = if r.greater_equal()? {
            0x114d84_u32
        } else {
            0x114d48_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_7b_unit::combat_body_block_7b;

mod combat_body_block_7c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_7c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114d48: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 00114d4e: movea.w (a0, d4.w * 4), a1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(4, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(1, value);
        // 00114d52: move.l $442780.l, d0
        e.step()?;
        let value_address = 0x442780_u32;
        let value = e.l(value_address)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114d58: add.l a1, d0
        e.step()?;
        let source = r.address(1)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00114d5a: move.w d0, $443722.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x443722_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114d60: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 00114d66: movea.w $2(a0, d4.w * 4), a1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(4, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(1, value);
        // 00114d6a: move.l $442784.l, d0
        e.step()?;
        let value_address = 0x442784_u32;
        let value = e.l(value_address)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114d70: add.l a1, d0
        e.step()?;
        let source = r.address(1)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00114d72: move.w d0, $443724.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x443724_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114d78: move.w #$3, $443726.l
        e.step()?;
        let value = 0x3_u32;
        let destination_address = 0x443726_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114d80: bra.w $114fba
        e.step()?;
        pc = 0x114fba_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_7c_unit::combat_body_block_7c;

mod combat_body_block_7d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_7d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114d84: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00114d86: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114d88: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00114d8e: movea.w (a0, d0.w * 4), a1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(1, value);
        // 00114d92: move.l $442780.l, d0
        e.step()?;
        let value_address = 0x442780_u32;
        let value = e.l(value_address)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114d98: add.l a1, d0
        e.step()?;
        let source = r.address(1)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00114d9a: move.w d0, $443722.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x443722_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114da0: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00114da2: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114da4: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00114daa: movea.w $2(a0, d0.w * 4), a1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(1, value);
        // 00114dae: move.l $442784.l, d0
        e.step()?;
        let value_address = 0x442784_u32;
        let value = e.l(value_address)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114db4: add.l a1, d0
        e.step()?;
        let source = r.address(1)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00114db6: move.w d0, $443724.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x443724_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114dbc: move.w #$3, $443726.l
        e.step()?;
        let value = 0x3_u32;
        let destination_address = 0x443726_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114dc4: bra.w $114fba
        e.step()?;
        pc = 0x114fba_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_7d_unit::combat_body_block_7d;

mod combat_body_block_7e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_7e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114dc8: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x114dca_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_7e_unit::combat_body_block_7e;

mod combat_body_block_7f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_7f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114dca: cmpi.w #$64, d3
        e.step()?;
        let source = 0x64_u32;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114dce: blt.w $114d18
        e.step()?;
        pc = if r.less()? {
            0x114d18_u32
        } else {
            0x114dd2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_7f_unit::combat_body_block_7f;

mod combat_body_block_80_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_80(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114dd2: bra.w $11510a
        e.step()?;
        pc = 0x11510a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_80_unit::combat_body_block_80;

mod combat_body_block_81_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_81(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114dd6: cmpi.b #$1, $44374f.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x44374f_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00114dde: ble.w $114ef2
        e.step()?;
        pc = if r.less_equal()? {
            0x114ef2_u32
        } else {
            0x114de2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_81_unit::combat_body_block_81;

mod combat_body_block_82_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_82(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114de2: move.w $4436b4.l, d0
        e.step()?;
        let value_address = 0x4436b4_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114de8: cmp.w $4435bc.l, d0
        e.step()?;
        let source_address = 0x4435bc_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114dee: bge.w $114ef2
        e.step()?;
        pc = if r.greater_equal()? {
            0x114ef2_u32
        } else {
            0x114df2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_82_unit::combat_body_block_82;

mod combat_body_block_83_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_83(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114df2: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114df8: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114dfc: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114e02: move.w $e(a0, d0.l), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114e06: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00114e08: move.b $44376d.l, d0
        e.step()?;
        let value_address = 0x44376d_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114e0e: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114e10: bsr.l $1363e4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114e16_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1363e4_u32)?;
        // 00114e16: move.w d0, d4
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(4, 16, value);
        r.move_flags(value, 16);
        // 00114e18: tst.w d4
        e.step()?;
        let value = r.data(4, 16)?;
        r.move_flags(value, 16);
        // 00114e1a: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00114e1c: blt.w $11510a
        e.step()?;
        pc = if r.less()? {
            0x11510a_u32
        } else {
            0x114e20_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_83_unit::combat_body_block_83;

mod combat_body_block_84_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_84(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114e20: cmpi.w #$9, d4
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114e24: bge.b $114e88
        e.step()?;
        pc = if r.greater_equal()? {
            0x114e88_u32
        } else {
            0x114e26_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_84_unit::combat_body_block_84;

mod combat_body_block_85_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_85(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114e26: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00114e28: bra.b $114e7c
        e.step()?;
        pc = 0x114e7c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_85_unit::combat_body_block_85;

mod combat_body_block_86_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_86(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114e2a: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 00114e30: movea.w (a0, d4.w * 4), a1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(4, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(1, value);
        // 00114e34: move.l $442780.l, d0
        e.step()?;
        let value_address = 0x442780_u32;
        let value = e.l(value_address)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114e3a: add.l a1, d0
        e.step()?;
        let source = r.address(1)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00114e3c: moveq #$6, d1
        e.step()?;
        r.put_data(1, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 00114e3e: mulu.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00114e40: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 00114e46: move.w d0, (a0, d1.l)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114e4a: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 00114e50: movea.w $2(a0, d4.w * 4), a1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(4, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(1, value);
        // 00114e54: move.l $442784.l, d0
        e.step()?;
        let value_address = 0x442784_u32;
        let value = e.l(value_address)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114e5a: add.l a1, d0
        e.step()?;
        let source = r.address(1)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00114e5c: moveq #$6, d1
        e.step()?;
        r.put_data(1, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 00114e5e: mulu.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00114e60: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 00114e66: move.w d0, $2(a0, d1.l)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114e6a: moveq #$6, d0
        e.step()?;
        r.put_data(0, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 00114e6c: mulu.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114e6e: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 00114e74: move.w #$3, $4(a0, d0.l)
        e.step()?;
        let value = 0x3_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x4_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114e7a: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x114e7c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_86_unit::combat_body_block_86;

mod combat_body_block_87_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_87(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114e7c: cmp.w $4435c0.l, d3
        e.step()?;
        let source_address = 0x4435c0_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114e82: blt.b $114e2a
        e.step()?;
        pc = if r.less()? {
            0x114e2a_u32
        } else {
            0x114e84_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_87_unit::combat_body_block_87;

mod combat_body_block_88_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_88(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114e84: bra.w $114fba
        e.step()?;
        pc = 0x114fba_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_88_unit::combat_body_block_88;

mod combat_body_block_89_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_89(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114e88: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00114e8a: bra.b $114ee6
        e.step()?;
        pc = 0x114ee6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_89_unit::combat_body_block_89;
mod combat_body_block_8a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_8a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114e8c: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00114e8e: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114e90: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00114e96: movea.w (a0, d0.w * 4), a1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(1, value);
        // 00114e9a: move.l $442780.l, d0
        e.step()?;
        let value_address = 0x442780_u32;
        let value = e.l(value_address)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114ea0: add.l a1, d0
        e.step()?;
        let source = r.address(1)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00114ea2: moveq #$6, d1
        e.step()?;
        r.put_data(1, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 00114ea4: mulu.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00114ea6: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 00114eac: move.w d0, (a0, d1.l)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114eb0: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00114eb2: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114eb4: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00114eba: movea.w $2(a0, d0.w * 4), a1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(1, value);
        // 00114ebe: move.l $442784.l, d0
        e.step()?;
        let value_address = 0x442784_u32;
        let value = e.l(value_address)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114ec4: add.l a1, d0
        e.step()?;
        let source = r.address(1)?;
        let destination = r.data(0, 32)?;
        let value = r.add_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00114ec6: moveq #$6, d1
        e.step()?;
        r.put_data(1, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 00114ec8: mulu.w d3, d1
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00114eca: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 00114ed0: move.w d0, $2(a0, d1.l)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114ed4: moveq #$6, d0
        e.step()?;
        r.put_data(0, 32, 0x6_u32);
        r.move_flags(0x6_u32, 32);
        // 00114ed6: mulu.w d3, d0
        e.step()?;
        let source = r.data(3, 16)?;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114ed8: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 00114ede: move.w #$3, $4(a0, d0.l)
        e.step()?;
        let value = 0x3_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x4_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114ee4: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x114ee6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_8a_unit::combat_body_block_8a;

mod combat_body_block_8b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_8b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114ee6: cmp.w $4435c0.l, d3
        e.step()?;
        let source_address = 0x4435c0_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114eec: blt.b $114e8c
        e.step()?;
        pc = if r.less()? {
            0x114e8c_u32
        } else {
            0x114eee_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_8b_unit::combat_body_block_8b;

mod combat_body_block_8c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_8c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114eee: bra.w $114fba
        e.step()?;
        pc = 0x114fba_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_8c_unit::combat_body_block_8c;

mod combat_body_block_8d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_8d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114ef2: cmpi.b #$2, $44374f.l
        e.step()?;
        let source = 0x2_u32;
        let destination_address = 0x44374f_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00114efa: bge.w $11510a
        e.step()?;
        pc = if r.greater_equal()? {
            0x11510a_u32
        } else {
            0x114efe_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_8d_unit::combat_body_block_8d;

mod combat_body_block_8e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_8e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114efe: move.w $4436b4.l, d0
        e.step()?;
        let value_address = 0x4436b4_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114f04: cmp.w $4435bc.l, d0
        e.step()?;
        let source_address = 0x4435bc_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114f0a: bgt.w $11510a
        e.step()?;
        pc = if r.greater()? {
            0x11510a_u32
        } else {
            0x114f0e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_8e_unit::combat_body_block_8e;

mod combat_body_block_8f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_8f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114f0e: clr.w $4435c0.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4435c0_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114f14: bra.w $114fb2
        e.step()?;
        pc = 0x114fb2_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_8f_unit::combat_body_block_8f;

mod combat_body_block_90_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_90(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114f18: addq.b #$1, d5
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(5, 8)?;
        let value = r.add_flags(destination, source, 8, true);
        r.put_data(5, 8, value);
        // 00114f1a: cmpi.b #$64, d5
        e.step()?;
        let source = 0x64_u32;
        let destination = r.data(5, 8)?;
        r.subtract_flags(destination, source, 8, false);
        // 00114f1e: ble.b $114f30
        e.step()?;
        pc = if r.less_equal()? {
            0x114f30_u32
        } else {
            0x114f20_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_90_unit::combat_body_block_90;

mod combat_body_block_91_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_91(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114f20: moveq #$0, d5
        e.step()?;
        r.put_data(5, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00114f22: tst.w $4435c0.l
        e.step()?;
        let value_address = 0x4435c0_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00114f28: bne.w $114fba
        e.step()?;
        pc = if r.not_equal()? {
            0x114fba_u32
        } else {
            0x114f2c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_91_unit::combat_body_block_91;

mod combat_body_block_92_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_92(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114f2c: bra.w $11510a
        e.step()?;
        pc = 0x11510a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_92_unit::combat_body_block_92;

mod combat_body_block_93_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_93(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114f30: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 00114f32: add.w $43874a.l, d0
        e.step()?;
        let source_address = 0x43874a_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114f38: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114f3a: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x114f40_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 00114f40: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00114f42: move.b d0, d6
        e.step()?;
        let value = r.data(0, 8)?;
        r.put_data(6, 8, value);
        r.move_flags(value, 8);
        // 00114f44: move.b d6, d0
        e.step()?;
        let value = r.data(6, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114f46: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114f48: lea.l $4435d6.l, a0
        e.step()?;
        let value = 0x4435d6_u32;
        r.put_address(0, value);
        // 00114f4e: move.w (a0, d0.l * 2), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114f52: cmp.w $4435bc.l, d0
        e.step()?;
        let source_address = 0x4435bc_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00114f58: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00114f5a: bgt.b $114f18
        e.step()?;
        pc = if r.greater()? {
            0x114f18_u32
        } else {
            0x114f5c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_93_unit::combat_body_block_93;

mod combat_body_block_94_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_94(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114f5c: move.b d6, d0
        e.step()?;
        let value = r.data(6, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114f5e: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114f60: move.b d7, d1
        e.step()?;
        let value = r.data(7, 8)?;
        r.put_data(1, 8, value);
        r.move_flags(value, 8);
        // 00114f62: ext.w d1
        e.step()?;
        let source = r.data(1, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00114f64: mulu.w #$6, d1
        e.step()?;
        let source = 0x6_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00114f68: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 00114f6e: move.w d0, (a0, d1.l)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114f72: cmpi.b #$9, d6
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(6, 8)?;
        r.subtract_flags(destination, source, 8, false);
        // 00114f76: bge.b $114f90
        e.step()?;
        pc = if r.greater_equal()? {
            0x114f90_u32
        } else {
            0x114f78_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_94_unit::combat_body_block_94;

mod combat_body_block_95_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_95(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114f78: move.b d7, d0
        e.step()?;
        let value = r.data(7, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114f7a: addq.b #$1, d7
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(7, 8)?;
        let value = r.add_flags(destination, source, 8, true);
        r.put_data(7, 8, value);
        // 00114f7c: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114f7e: mulu.w #$6, d0
        e.step()?;
        let source = 0x6_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114f82: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 00114f88: move.w #$1, $4(a0, d0.l)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x4_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00114f8e: bra.b $114fa6
        e.step()?;
        pc = 0x114fa6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_95_unit::combat_body_block_95;

mod combat_body_block_96_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_96(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114f90: move.b d7, d0
        e.step()?;
        let value = r.data(7, 8)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00114f92: addq.b #$1, d7
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(7, 8)?;
        let value = r.add_flags(destination, source, 8, true);
        r.put_data(7, 8, value);
        // 00114f94: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114f96: mulu.w #$6, d0
        e.step()?;
        let source = 0x6_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114f9a: lea.l $443722.l, a0
        e.step()?;
        let value = 0x443722_u32;
        r.put_address(0, value);
        // 00114fa0: move.w #$2, $4(a0, d0.l)
        e.step()?;
        let value = 0x2_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x4_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x114fa6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_96_unit::combat_body_block_96;

mod combat_body_block_97_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_97(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114fa6: subq.b #$1, -$2(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
        let destination = u32::from(e.b(destination_address)?);
        let value = r.subtract_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        // 00114faa: addq.w #$1, $4435c0.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4435c0_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00114fb0: moveq #$0, d5
        e.step()?;
        r.put_data(5, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        pc = 0x114fb2_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_97_unit::combat_body_block_97;

mod combat_body_block_98_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_98(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114fb2: tst.b -$2(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffe_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00114fb6: bne.w $114f18
        e.step()?;
        pc = if r.not_equal()? {
            0x114f18_u32
        } else {
            0x114fba_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_98_unit::combat_body_block_98;

mod combat_body_block_99_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_99(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00114fba: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114fc0: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114fc4: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114fca: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00114fcc: subq.b #$2, $109(a0)
        e.step()?;
        let source = 0x2_u32;
        let destination_address = r.address(0)?.wrapping_add(0x109_u32);
        let destination = u32::from(e.b(destination_address)?);
        let value = r.subtract_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        // 00114fd0: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114fd6: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114fda: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114fe0: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00114fe2: subq.b #$3, $106(a0)
        e.step()?;
        let source = 0x3_u32;
        let destination_address = r.address(0)?.wrapping_add(0x106_u32);
        let destination = u32::from(e.b(destination_address)?);
        let value = r.subtract_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        // 00114fe6: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00114fec: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00114ff0: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00114ff6: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00114ff8: cmpi.b #$1, $109(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x109_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00114ffe: bge.b $115018
        e.step()?;
        pc = if r.greater_equal()? {
            0x115018_u32
        } else {
            0x115000_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_99_unit::combat_body_block_99;
mod combat_body_block_9a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_9a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115000: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115006: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011500a: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115010: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115012: clr.b $104(a0)
        e.step()?;
        let value = 0;
        let destination_address = r.address(0)?.wrapping_add(0x104_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115016: bra.b $115030
        e.step()?;
        pc = 0x115030_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_9a_unit::combat_body_block_9a;

mod combat_body_block_9b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_9b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115018: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011501e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115022: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115028: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0011502a: move.b #$1, $104(a0)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x104_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x115030_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_9b_unit::combat_body_block_9b;

mod combat_body_block_9c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_9c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115030: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00115032: move.b $44376c.l, d0
        e.step()?;
        let value_address = 0x44376c_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00115038: muls.w $4435cc.l, d0
        e.step()?;
        let source_address = 0x4435cc_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011503e: move.w $447196.l, d1
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00115044: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00115048: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0011504e: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115050: sub.w d0, $10e(a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x10e_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00115054: move.w #$1, $438760.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x438760_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011505c: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 0011505e: add.w $447196.l, d0
        e.step()?;
        let source_address = 0x447196_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115064: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115066: bsr.l $11f1a8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11506c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x11f1a8_u32)?;
        // 0011506c: cmpi.b #$9, $443761.l
        e.step()?;
        let source = 0x9_u32;
        let destination_address = 0x443761_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00115074: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00115076: bne.b $115098
        e.step()?;
        pc = if r.not_equal()? {
            0x115098_u32
        } else {
            0x115078_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_9c_unit::combat_body_block_9c;

mod combat_body_block_9d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_9d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115078: move.w $443722.l, -(a7)
        e.step()?;
        let value_address = 0x443722_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011507e: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 00115080: add.w $447196.l, d0
        e.step()?;
        let source_address = 0x447196_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115086: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115088: bsr.l $15edfa
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11508e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x15edfa_u32)?;
        // 0011508e: move.w #$1, $4435cc.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x4435cc_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115096: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x115098_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_9d_unit::combat_body_block_9d;

mod combat_body_block_9e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_9e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115098: bsr.l $15a1f6
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11509e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x15a1f6_u32)?;
        // 0011509e: bsr.l $159192
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1150a4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x159192_u32)?;
        // 001150a4: tst.b $443755.l
        e.step()?;
        let value_address = 0x443755_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001150aa: beq.b $1150c0
        e.step()?;
        pc = if r.equal()? {
            0x1150c0_u32
        } else {
            0x1150ac_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_9e_unit::combat_body_block_9e;

mod combat_body_block_9f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_9f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001150ac: move.w #$4, -(a7)
        e.step()?;
        let value = 0x4_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001150b0: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1150b6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 001150b6: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001150b8: move.w d0, $44331a.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44331a_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001150be: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1150c0_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_9f_unit::combat_body_block_9f;

mod combat_body_block_a0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_a0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001150c0: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 001150c2: add.w $447196.l, d0
        e.step()?;
        let source_address = 0x447196_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001150c8: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001150ca: move.w $4435c0.l, -(a7)
        e.step()?;
        let value_address = 0x4435c0_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001150d0: bsr.l $10d4fe
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1150d6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x10d4fe_u32)?;
        // 001150d6: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1150d8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_a0_unit::combat_body_block_a0;

mod combat_body_block_a1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_a1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001150d8: cmpi.b #$9, $443761.l
        e.step()?;
        let source = 0x9_u32;
        let destination_address = 0x443761_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 001150e0: beq.w $1145c8
        e.step()?;
        pc = if r.equal()? {
            0x1145c8_u32
        } else {
            0x1150e4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_a1_unit::combat_body_block_a1;

mod combat_body_block_a2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_a2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001150e4: move.b #$1, -$6(a6)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001150ea: addq.b #$1, -$c(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        let destination = u32::from(e.b(destination_address)?);
        let value = r.add_flags(destination, source, 8, true);
        r.write_memory(e, destination_address, 8, value)?;
        pc = 0x1150ee_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_a2_unit::combat_body_block_a2;

mod combat_body_block_a3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_a3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001150ee: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001150f4: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001150f8: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001150fe: move.b -$c(a6), d1
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff4_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(1, 8, value);
        r.move_flags(value, 8);
        // 00115102: cmp.b $7e(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x7e_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.b(source_address)?);
        let destination = r.data(1, 8)?;
        r.subtract_flags(destination, source, 8, false);
        // 00115106: blt.w $114852
        e.step()?;
        pc = if r.less()? {
            0x114852_u32
        } else {
            0x11510a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_a3_unit::combat_body_block_a3;

mod combat_body_block_a4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_a4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011510a: addq.w #$1, $438780.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x438780_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00115110: bsr.l $13822a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115116_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13822a_u32)?;
        // 00115116: tst.b -$6(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffa_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0011511a: bne.w $115732
        e.step()?;
        pc = if r.not_equal()? {
            0x115732_u32
        } else {
            0x11511e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_a4_unit::combat_body_block_a4;

mod combat_body_block_a5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_a5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011511e: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115124: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115128: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0011512e: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115130: tst.b $b3(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xb3_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00115134: bne.b $115140
        e.step()?;
        pc = if r.not_equal()? {
            0x115140_u32
        } else {
            0x115136_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_a5_unit::combat_body_block_a5;

mod combat_body_block_a6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_a6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115136: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115138: bsr.l $110e96
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11513e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x110e96_u32)?;
        // 0011513e: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x115140_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_a6_unit::combat_body_block_a6;

mod combat_body_block_a7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_a7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115140: clr.w -$4(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115144: move.w $4435c4.l, d0
        e.step()?;
        let value_address = 0x4435c4_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011514a: sub.w $44374c.l, d0
        e.step()?;
        let source_address = 0x44374c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115150: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00115152: ble.w $115732
        e.step()?;
        pc = if r.less_equal()? {
            0x115732_u32
        } else {
            0x115156_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_a7_unit::combat_body_block_a7;

mod combat_body_block_a8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_a8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115156: bra.w $115664
        e.step()?;
        pc = 0x115664_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_a8_unit::combat_body_block_a8;

mod combat_body_block_a9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_a9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011515a: move.w $4435c4.l, d0
        e.step()?;
        let value_address = 0x4435c4_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115160: sub.w $44374c.l, d0
        e.step()?;
        let source_address = 0x44374c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115166: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00115168: ble.w $115732
        e.step()?;
        pc = if r.less_equal()? {
            0x115732_u32
        } else {
            0x11516c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_a9_unit::combat_body_block_a9;

mod combat_body_block_aa_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_aa(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011516c: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115172: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115176: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0011517c: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0011517e: move.w $10c(a0), d4
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x10c_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(4, 16, value);
        r.move_flags(value, 16);
        // 00115182: cmpi.w #$9, d4
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00115186: bge.b $1151c8
        e.step()?;
        pc = if r.greater_equal()? {
            0x1151c8_u32
        } else {
            0x115188_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_aa_unit::combat_body_block_aa;

mod combat_body_block_ab_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_ab(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115188: cmpi.w #$ffff, d4
        e.step()?;
        let source = 0xffff_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0011518c: ble.b $1151c8
        e.step()?;
        pc = if r.less_equal()? {
            0x1151c8_u32
        } else {
            0x11518e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_ab_unit::combat_body_block_ab;

mod combat_body_block_ac_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_ac(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011518e: move.w d4, d0
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115190: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115194: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0011519a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0011519c: tst.b $9df(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x9df_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001151a0: beq.b $115214
        e.step()?;
        pc = if r.equal()? {
            0x115214_u32
        } else {
            0x1151a2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_ac_unit::combat_body_block_ac;

mod combat_body_block_ad_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_ad(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001151a2: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001151a8: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001151ac: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001151b2: move.w d4, d1
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001151b4: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 001151b8: lea.l $448aa0.l, a1
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(1, value);
        // 001151be: move.w $10(a1, d1.l), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001151c2: cmp.w $e(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001151c6: beq.b $115214
        e.step()?;
        pc = if r.equal()? {
            0x115214_u32
        } else {
            0x1151c8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_ad_unit::combat_body_block_ad;

mod combat_body_block_ae_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_ae(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001151c8: cmpi.w #$9, d4
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001151cc: ble.b $11520e
        e.step()?;
        pc = if r.less_equal()? {
            0x11520e_u32
        } else {
            0x1151ce_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_ae_unit::combat_body_block_ae;

mod combat_body_block_af_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_af(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001151ce: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001151d0: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001151d2: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001151d6: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001151dc: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001151de: cmpi.w #$1, $110(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x110_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001151e4: blt.b $115214
        e.step()?;
        pc = if r.less()? {
            0x115214_u32
        } else {
            0x1151e6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_af_unit::combat_body_block_af;

mod combat_body_block_b0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_b0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001151e6: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001151e8: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001151ea: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001151ee: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001151f4: move.w $447196.l, d1
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001151fa: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 001151fe: lea.l $439d3c.l, a1
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(1, value);
        // 00115204: move.w $e(a1, d1.l), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00115208: cmp.w $e(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0011520c: beq.b $115214
        e.step()?;
        pc = if r.equal()? {
            0x115214_u32
        } else {
            0x11520e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_b0_unit::combat_body_block_b0;

mod combat_body_block_b1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_b1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011520e: tst.w d4
        e.step()?;
        let value = r.data(4, 16)?;
        r.move_flags(value, 16);
        // 00115210: bge.w $115408
        e.step()?;
        pc = if r.greater_equal()? {
            0x115408_u32
        } else {
            0x115214_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_b1_unit::combat_body_block_b1;

mod combat_body_block_b2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_b2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115214: cmpi.b #$1, -$5(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 0011521a: beq.b $115232
        e.step()?;
        pc = if r.equal()? {
            0x115232_u32
        } else {
            0x11521c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_b2_unit::combat_body_block_b2;

mod combat_body_block_b3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_b3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011521c: moveq #$a, d0
        e.step()?;
        r.put_data(0, 32, 0xa_u32);
        r.move_flags(0xa_u32, 32);
        // 0011521e: add.w $43874a.l, d0
        e.step()?;
        let source_address = 0x43874a_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115224: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115226: bsr.l $153ca4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11522c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153ca4_u32)?;
        // 0011522c: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011522e: move.w d0, d4
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(4, 16, value);
        r.move_flags(value, 16);
        // 00115230: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x115232_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_b3_unit::combat_body_block_b3;

mod combat_body_block_b4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_b4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115232: move.b $44273a.l, d0
        e.step()?;
        let value_address = 0x44273a_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00115238: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011523a: cmp.w d0, d4
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0011523c: ble.b $11526c
        e.step()?;
        pc = if r.less_equal()? {
            0x11526c_u32
        } else {
            0x11523e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_b4_unit::combat_body_block_b4;

mod combat_body_block_b5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_b5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011523e: cmpi.w #$a, d4
        e.step()?;
        let source = 0xa_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00115242: bge.b $11526c
        e.step()?;
        pc = if r.greater_equal()? {
            0x11526c_u32
        } else {
            0x115244_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_b5_unit::combat_body_block_b5;

mod combat_body_block_b6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_b6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115244: tst.b -$5(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00115248: beq.b $115214
        e.step()?;
        pc = if r.equal()? {
            0x115214_u32
        } else {
            0x11524a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_b6_unit::combat_body_block_b6;

mod combat_body_block_b7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_b7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011524a: move.w -$4(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011524e: addq.w #$1, -$4(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00115252: move.w d0, d4
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(4, 16, value);
        r.move_flags(value, 16);
        // 00115254: move.w $447196.l, d1
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0011525a: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0011525e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115264: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115266: move.w d0, $10c(a0)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x10c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011526a: bra.b $115214
        e.step()?;
        pc = 0x115214_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_b7_unit::combat_body_block_b7;

mod combat_body_block_b8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_b8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011526c: cmpi.w #$a, d4
        e.step()?;
        let source = 0xa_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00115270: bge.w $115310
        e.step()?;
        pc = if r.greater_equal()? {
            0x115310_u32
        } else {
            0x115274_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_b8_unit::combat_body_block_b8;

mod combat_body_block_b9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_b9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115274: cmpi.w #$ffff, d4
        e.step()?;
        let source = 0xffff_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00115278: ble.w $115310
        e.step()?;
        pc = if r.less_equal()? {
            0x115310_u32
        } else {
            0x11527c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_b9_unit::combat_body_block_b9;

mod combat_body_block_ba_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_ba(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011527c: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115282: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115286: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0011528c: move.w d4, d1
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0011528e: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00115292: lea.l $448aa0.l, a1
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(1, value);
        // 00115298: move.w $10(a1, d1.l), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0011529c: cmp.w $e(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001152a0: bne.b $1152ce
        e.step()?;
        pc = if r.not_equal()? {
            0x1152ce_u32
        } else {
            0x1152a2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_ba_unit::combat_body_block_ba;

mod combat_body_block_bb_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_bb(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001152a2: tst.b -$5(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001152a6: beq.w $115214
        e.step()?;
        pc = if r.equal()? {
            0x115214_u32
        } else {
            0x1152aa_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_bb_unit::combat_body_block_bb;

mod combat_body_block_bc_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_bc(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001152aa: move.w -$4(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001152ae: addq.w #$1, -$4(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001152b2: move.w d0, d4
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(4, 16, value);
        r.move_flags(value, 16);
        // 001152b4: move.w $447196.l, d1
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001152ba: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 001152be: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001152c4: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001152c6: move.w d0, $10c(a0)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x10c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001152ca: bra.w $115214
        e.step()?;
        pc = 0x115214_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_bc_unit::combat_body_block_bc;

mod combat_body_block_bd_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_bd(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001152ce: move.w d4, d0
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001152d0: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001152d4: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001152da: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001152dc: tst.b $9df(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x9df_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001152e0: bne.w $1153f2
        e.step()?;
        pc = if r.not_equal()? {
            0x1153f2_u32
        } else {
            0x1152e4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_bd_unit::combat_body_block_bd;

mod combat_body_block_be_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_be(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001152e4: tst.b -$5(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001152e8: beq.w $115214
        e.step()?;
        pc = if r.equal()? {
            0x115214_u32
        } else {
            0x1152ec_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_be_unit::combat_body_block_be;

mod combat_body_block_bf_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_bf(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001152ec: move.w -$4(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001152f0: addq.w #$1, -$4(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001152f4: move.w d0, d4
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(4, 16, value);
        r.move_flags(value, 16);
        // 001152f6: move.w $447196.l, d1
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001152fc: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00115300: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115306: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115308: move.w d0, $10c(a0)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x10c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011530c: bra.w $115214
        e.step()?;
        pc = 0x115214_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_bf_unit::combat_body_block_bf;

mod combat_body_block_c0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_c0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115310: cmpi.w #$9, d4
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(4, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00115314: ble.w $1153f2
        e.step()?;
        pc = if r.less_equal()? {
            0x1153f2_u32
        } else {
            0x115318_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_c0_unit::combat_body_block_c0;

mod combat_body_block_c1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_c1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115318: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011531e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115322: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115328: moveq #$f6, d1
        e.step()?;
        r.put_data(1, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0011532a: add.w d4, d1
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0011532c: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00115330: lea.l $439d3c.l, a1
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(1, value);
        // 00115336: move.w $e(a1, d1.l), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0011533a: cmp.w $e(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0xe_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0011533e: bne.b $11536c
        e.step()?;
        pc = if r.not_equal()? {
            0x11536c_u32
        } else {
            0x115340_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_c1_unit::combat_body_block_c1;

mod combat_body_block_c2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_c2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115340: tst.b -$5(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00115344: beq.w $115214
        e.step()?;
        pc = if r.equal()? {
            0x115214_u32
        } else {
            0x115348_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_c2_unit::combat_body_block_c2;

mod combat_body_block_c3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_c3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115348: move.w -$4(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011534c: addq.w #$1, -$4(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00115350: move.w d0, d4
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(4, 16, value);
        r.move_flags(value, 16);
        // 00115352: move.w $447196.l, d1
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00115358: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0011535c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115362: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115364: move.w d0, $10c(a0)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x10c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115368: bra.w $115214
        e.step()?;
        pc = 0x115214_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_c3_unit::combat_body_block_c3;

mod combat_body_block_c4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_c4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011536c: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 0011536e: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115370: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115374: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0011537a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0011537c: cmpi.w #$1, $110(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x110_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00115382: bge.b $1153b0
        e.step()?;
        pc = if r.greater_equal()? {
            0x1153b0_u32
        } else {
            0x115384_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_c4_unit::combat_body_block_c4;

mod combat_body_block_c5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_c5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115384: tst.b -$5(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00115388: beq.w $115214
        e.step()?;
        pc = if r.equal()? {
            0x115214_u32
        } else {
            0x11538c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_c5_unit::combat_body_block_c5;

mod combat_body_block_c6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_c6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011538c: move.w -$4(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115390: addq.w #$1, -$4(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00115394: move.w d0, d4
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(4, 16, value);
        r.move_flags(value, 16);
        // 00115396: move.w $447196.l, d1
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0011539c: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 001153a0: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001153a6: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001153a8: move.w d0, $10c(a0)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x10c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001153ac: bra.w $115214
        e.step()?;
        pc = 0x115214_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_c6_unit::combat_body_block_c6;

mod combat_body_block_c7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_c7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001153b0: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 001153b2: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001153b4: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001153b8: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001153be: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001153c0: tst.b $82(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x82_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001153c4: beq.b $1153f2
        e.step()?;
        pc = if r.equal()? {
            0x1153f2_u32
        } else {
            0x1153c6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_c7_unit::combat_body_block_c7;

mod combat_body_block_c8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_c8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001153c6: tst.b -$5(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001153ca: beq.w $115214
        e.step()?;
        pc = if r.equal()? {
            0x115214_u32
        } else {
            0x1153ce_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_c8_unit::combat_body_block_c8;

mod combat_body_block_c9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_c9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001153ce: move.w -$4(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001153d2: addq.w #$1, -$4(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001153d6: move.w d0, d4
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(4, 16, value);
        r.move_flags(value, 16);
        // 001153d8: move.w $447196.l, d1
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001153de: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 001153e2: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001153e8: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001153ea: move.w d0, $10c(a0)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x10c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001153ee: bra.w $115214
        e.step()?;
        pc = 0x115214_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_c9_unit::combat_body_block_c9;

mod combat_body_block_ca_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_ca(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001153f2: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001153f8: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001153fc: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115402: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115404: move.w d4, $10c(a0)
        e.step()?;
        let value = r.data(4, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x10c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x115408_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_ca_unit::combat_body_block_ca;

mod combat_body_block_cb_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_cb(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115408: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 0011540e: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115414: move.w (a0, d0.w * 4), -$e(a6)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011541a: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 00115420: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115426: move.w $2(a0, d0.w * 4), -$10(a6)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011542c: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115432: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115436: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0011543c: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0011543e: cmpi.w #$a, $10c(a0)
        e.step()?;
        let source = 0xa_u32;
        let destination_address = r.address(0)?.wrapping_add(0x10c_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00115444: bge.b $115460
        e.step()?;
        pc = if r.greater_equal()? {
            0x115460_u32
        } else {
            0x115446_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_cb_unit::combat_body_block_cb;

mod combat_body_block_cc_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_cc(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115446: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 0011544c: move.w (a0, d4.w * 4), -$8(a6)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(4, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115452: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 00115458: move.w $2(a0, d4.w * 4), -$a(a6)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(4, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011545e: bra.b $115480
        e.step()?;
        pc = 0x115480_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_cc_unit::combat_body_block_cc;

mod combat_body_block_cd_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_cd(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115460: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00115462: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115464: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 0011546a: move.w (a0, d0.w * 4), -$8(a6)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115470: moveq #$f6, d0
        e.step()?;
        r.put_data(0, 32, 0xfffffff6_u32);
        r.move_flags(0xfffffff6_u32, 32);
        // 00115472: add.w d4, d0
        e.step()?;
        let source = r.data(4, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115474: lea.l $446f0a.l, a0
        e.step()?;
        let value = 0x446f0a_u32;
        r.put_address(0, value);
        // 0011547a: move.w $2(a0, d0.w * 4), -$a(a6)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x115480_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_cd_unit::combat_body_block_cd;

mod combat_body_block_ce_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_ce(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115480: move.w -$a(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff6_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115484: move.w -$8(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff8_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115488: move.w -$10(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff0_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011548c: move.w -$e(a6), -(a7)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff2_u32);
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115490: bsr.l $10ae90
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115496_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x10ae90_u32)?;
        // 00115496: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 00115498: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0011549a: bne.b $1154f0
        e.step()?;
        pc = if r.not_equal()? {
            0x1154f0_u32
        } else {
            0x11549c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_ce_unit::combat_body_block_ce;

mod combat_body_block_cf_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_cf(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011549c: cmpi.w #$6e, -$4(a6)
        e.step()?;
        let source = 0x6e_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001154a2: bge.b $1154f0
        e.step()?;
        pc = if r.greater_equal()? {
            0x1154f0_u32
        } else {
            0x1154a4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_cf_unit::combat_body_block_cf;

mod combat_body_block_d0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_d0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001154a4: move.w -$4(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001154a8: addq.w #$1, -$4(a6)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001154ac: move.w d0, d4
        e.step()?;
        let value = r.data(0, 16)?;
        r.put_data(4, 16, value);
        r.move_flags(value, 16);
        // 001154ae: move.w $447196.l, d1
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001154b4: mulu.w #$120, d1
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 001154b8: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001154be: adda.l d1, a0
        e.step()?;
        let source = r.data(1, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001154c0: move.w d0, $10c(a0)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x10c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001154c4: move.b #$1, -$5(a6)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001154ca: cmpi.w #$6e, -$4(a6)
        e.step()?;
        let source = 0x6e_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001154d0: ble.w $115232
        e.step()?;
        pc = if r.less_equal()? {
            0x115232_u32
        } else {
            0x1154d4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_d0_unit::combat_body_block_d0;

mod combat_body_block_d1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_d1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001154d4: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001154da: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001154de: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001154e4: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001154e6: move.w #$ffff, $10c(a0)
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = r.address(0)?.wrapping_add(0x10c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001154ec: bra.w $115232
        e.step()?;
        pc = 0x115232_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_d1_unit::combat_body_block_d1;

mod combat_body_block_d2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_d2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001154f0: clr.b -$5(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001154f4: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001154fa: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001154fe: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115504: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115506: cmpi.w #$6e, $10c(a0)
        e.step()?;
        let source = 0x6e_u32;
        let destination_address = r.address(0)?.wrapping_add(0x10c_u32);
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0011550c: ble.b $115526
        e.step()?;
        pc = if r.less_equal()? {
            0x115526_u32
        } else {
            0x11550e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_d2_unit::combat_body_block_d2;

mod combat_body_block_d3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_d3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011550e: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115514: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115518: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0011551e: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115520: move.w #$ffff, $10c(a0)
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = r.address(0)?.wrapping_add(0x10c_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x115526_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_d3_unit::combat_body_block_d3;

mod combat_body_block_d4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_d4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115526: bsr.l $13e276
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11552c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13e276_u32)?;
        // 0011552c: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 0011552e: bne.w $11560c
        e.step()?;
        pc = if r.not_equal()? {
            0x11560c_u32
        } else {
            0x115532_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_d4_unit::combat_body_block_d4;

mod combat_body_block_d5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_d5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115532: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115534: bsr.l $110e96
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11553a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x110e96_u32)?;
        // 0011553a: cmpi.w #$2, $438780.l
        e.step()?;
        let source = 0x2_u32;
        let destination_address = 0x438780_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00115542: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00115544: bge.w $115732
        e.step()?;
        pc = if r.greater_equal()? {
            0x115732_u32
        } else {
            0x115548_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_d5_unit::combat_body_block_d5;

mod combat_body_block_d6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_d6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115548: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011554e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115552: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115558: tst.w $42(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x42_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0011555c: beq.w $115732
        e.step()?;
        pc = if r.equal()? {
            0x115732_u32
        } else {
            0x115560_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_d6_unit::combat_body_block_d6;

mod combat_body_block_d7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_d7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115560: tst.w $43877e.l
        e.step()?;
        let value_address = 0x43877e_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00115566: bne.w $115732
        e.step()?;
        pc = if r.not_equal()? {
            0x115732_u32
        } else {
            0x11556a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_d7_unit::combat_body_block_d7;

mod combat_body_block_d8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_d8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011556a: addq.w #$2, $438780.l
        e.step()?;
        let source = 0x2_u32;
        let destination_address = 0x438780_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00115570: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115576: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011557a: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115580: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115582: clr.b $106(a0)
        e.step()?;
        let value = 0;
        let destination_address = r.address(0)?.wrapping_add(0x106_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115586: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011558c: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115590: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115596: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115598: tst.b $105(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x105_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0011559c: bne.w $115732
        e.step()?;
        pc = if r.not_equal()? {
            0x115732_u32
        } else {
            0x1155a0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_d8_unit::combat_body_block_d8;

mod combat_body_block_d9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_d9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001155a0: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001155a6: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001155aa: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001155b0: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001155b2: tst.b $b7(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xb7_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001155b6: bne.w $115732
        e.step()?;
        pc = if r.not_equal()? {
            0x115732_u32
        } else {
            0x1155ba_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_d9_unit::combat_body_block_d9;

mod combat_body_block_da_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_da(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001155ba: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001155c0: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001155c4: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001155ca: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001155cc: tst.b $cf(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xcf_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001155d0: bne.w $115732
        e.step()?;
        pc = if r.not_equal()? {
            0x115732_u32
        } else {
            0x1155d4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_da_unit::combat_body_block_da;

mod combat_body_block_db_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_db(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001155d4: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001155da: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001155de: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001155e4: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001155e6: tst.b $d9(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xd9_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001155ea: bne.w $115732
        e.step()?;
        pc = if r.not_equal()? {
            0x115732_u32
        } else {
            0x1155ee_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_db_unit::combat_body_block_db;

mod combat_body_block_dc_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_dc(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001155ee: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001155f4: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001155f8: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001155fe: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115600: tst.b $b3(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xb3_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00115604: beq.w $1147be
        e.step()?;
        pc = if r.equal()? {
            0x1147be_u32
        } else {
            0x115608_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_dc_unit::combat_body_block_dc;

mod combat_body_block_dd_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_dd(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115608: bra.w $115732
        e.step()?;
        pc = 0x115732_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_dd_unit::combat_body_block_dd;

mod combat_body_block_de_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_de(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011560c: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115612: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115616: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0011561c: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0011561e: tst.b $b3(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xb3_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00115622: bne.w $115732
        e.step()?;
        pc = if r.not_equal()? {
            0x115732_u32
        } else {
            0x115626_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_de_unit::combat_body_block_de;

mod combat_body_block_df_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_df(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115626: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115628: bsr.l $110e96
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11562e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x110e96_u32)?;
        // 0011562e: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 00115634: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011563a: tst.w (a0, d0.w * 2)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0011563e: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00115640: bge.b $11564c
        e.step()?;
        pc = if r.greater_equal()? {
            0x11564c_u32
        } else {
            0x115642_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_df_unit::combat_body_block_df;

mod combat_body_block_e0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_e0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115642: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115644: bsr.l $125a3a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11564a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x125a3a_u32)?;
        // 0011564a: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x11564c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_e0_unit::combat_body_block_e0;

mod combat_body_block_e1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_e1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011564c: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 00115652: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115658: move.w (a0, d0.w * 2), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011565c: bsr.l $11edc4
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115662_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x11edc4_u32)?;
        // 00115662: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x115664_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_e1_unit::combat_body_block_e1;

mod combat_body_block_e2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_e2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115664: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011566a: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011566e: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115674: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115676: tst.b $106(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x106_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0011567a: ble.b $1156b0
        e.step()?;
        pc = if r.less_equal()? {
            0x1156b0_u32
        } else {
            0x11567c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_e2_unit::combat_body_block_e2;

mod combat_body_block_e3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_e3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011567c: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115682: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115686: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 0011568c: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0011568e: cmpi.b #$1, $109(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x109_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00115694: ble.b $1156b0
        e.step()?;
        pc = if r.less_equal()? {
            0x1156b0_u32
        } else {
            0x115696_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_e3_unit::combat_body_block_e3;

mod combat_body_block_e4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_e4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115696: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011569c: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001156a0: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001156a6: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001156a8: tst.b $b3(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xb3_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001156ac: beq.w $11515a
        e.step()?;
        pc = if r.equal()? {
            0x11515a_u32
        } else {
            0x1156b0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_e4_unit::combat_body_block_e4;

mod combat_body_block_e5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_e5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001156b0: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001156b6: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001156ba: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001156c0: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001156c2: tst.b $b2(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0xb2_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001156c6: beq.b $1156d2
        e.step()?;
        pc = if r.equal()? {
            0x1156d2_u32
        } else {
            0x1156c8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_e5_unit::combat_body_block_e5;

mod combat_body_block_e6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_e6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001156c8: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001156ca: bsr.l $110e96
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1156d0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x110e96_u32)?;
        // 001156d0: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1156d2_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_e6_unit::combat_body_block_e6;

mod combat_body_block_e7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_e7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001156d2: cmpi.w #$2, $438780.l
        e.step()?;
        let source = 0x2_u32;
        let destination_address = 0x438780_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001156da: bge.b $115732
        e.step()?;
        pc = if r.greater_equal()? {
            0x115732_u32
        } else {
            0x1156dc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_e7_unit::combat_body_block_e7;

mod combat_body_block_e8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_e8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001156dc: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001156e2: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001156e6: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 001156ec: tst.w $42(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x42_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001156f0: beq.b $115732
        e.step()?;
        pc = if r.equal()? {
            0x115732_u32
        } else {
            0x1156f2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_e8_unit::combat_body_block_e8;

mod combat_body_block_e9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_e9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001156f2: tst.w $43877e.l
        e.step()?;
        let value_address = 0x43877e_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001156f8: bne.b $115732
        e.step()?;
        pc = if r.not_equal()? {
            0x115732_u32
        } else {
            0x1156fa_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_e9_unit::combat_body_block_e9;

mod combat_body_block_ea_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_ea(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001156fa: move.w #$2, $438780.l
        e.step()?;
        let value = 0x2_u32;
        let destination_address = 0x438780_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115702: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115708: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011570c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115712: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115714: clr.b $106(a0)
        e.step()?;
        let value = 0;
        let destination_address = r.address(0)?.wrapping_add(0x106_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115718: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011571e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115722: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115728: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0011572a: tst.b $105(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x105_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0011572e: beq.w $1147be
        e.step()?;
        pc = if r.equal()? {
            0x1147be_u32
        } else {
            0x115732_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_ea_unit::combat_body_block_ea;

mod combat_body_block_eb_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_eb(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115732: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115738: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011573c: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115742: tst.w $48(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x48_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00115746: beq.b $115776
        e.step()?;
        pc = if r.equal()? {
            0x115776_u32
        } else {
            0x115748_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_eb_unit::combat_body_block_eb;

mod combat_body_block_ec_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_ec(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115748: move.w $447196.l, d0
        e.step()?;
        let value_address = 0x447196_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011574e: mulu.w #$120, d0
        e.step()?;
        let source = 0x120_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115752: lea.l $439d3c.l, a0
        e.step()?;
        let value = 0x439d3c_u32;
        r.put_address(0, value);
        // 00115758: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0011575a: cmpi.b #$1, $109(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x109_u32);
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00115760: ble.b $115776
        e.step()?;
        pc = if r.less_equal()? {
            0x115776_u32
        } else {
            0x115762_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_ec_unit::combat_body_block_ec;

mod combat_body_block_ed_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_ed(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115762: cmpi.w #$2, $438780.l
        e.step()?;
        let source = 0x2_u32;
        let destination_address = 0x438780_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0011576a: bge.b $115776
        e.step()?;
        pc = if r.greater_equal()? {
            0x115776_u32
        } else {
            0x11576c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_ed_unit::combat_body_block_ed;

mod combat_body_block_ee_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_ee(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011576c: addq.w #$1, $438780.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x438780_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00115772: bra.w $1146ee
        e.step()?;
        pc = 0x1146ee_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_ee_unit::combat_body_block_ee;

mod combat_body_block_ef_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_ef(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115776: clr.w -$4(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffc_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011577a: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0011577c: move.b d0, -$5(a6)
        e.step()?;
        let value = r.data(0, 8)?;
        let destination_address = r.address(6)?.wrapping_add(0xfffffffb_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115780: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115782: bsr.l $125a3a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115788_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x125a3a_u32)?;
        // 00115788: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x11578a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_ef_unit::combat_body_block_ef;

mod combat_body_block_f0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_f0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011578a: tst.w $438764.l
        e.step()?;
        let value_address = 0x438764_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00115790: bne.b $1157c0
        e.step()?;
        pc = if r.not_equal()? {
            0x1157c0_u32
        } else {
            0x115792_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_f0_unit::combat_body_block_f0;

mod combat_body_block_f1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_f1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115792: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115798: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011579c: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001157a2: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001157a4: tst.w $90(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x90_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001157a8: bne.b $1157c0
        e.step()?;
        pc = if r.not_equal()? {
            0x1157c0_u32
        } else {
            0x1157aa_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_f1_unit::combat_body_block_f1;

mod combat_body_block_f2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_f2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001157aa: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001157b0: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001157b4: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001157ba: tst.w $5e(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x5e_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001157be: beq.b $115816
        e.step()?;
        pc = if r.equal()? {
            0x115816_u32
        } else {
            0x1157c0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_f2_unit::combat_body_block_f2;

mod combat_body_block_f3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_f3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001157c0: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 001157c6: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001157cc: cmpi.w #$9, (a0, d0.w * 2)
        e.step()?;
        let source = 0x9_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001157d2: bge.b $115816
        e.step()?;
        pc = if r.greater_equal()? {
            0x115816_u32
        } else {
            0x1157d4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_f3_unit::combat_body_block_f3;

mod combat_body_block_f4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_f4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001157d4: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 001157da: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001157e0: cmpi.w #$ffff, (a0, d0.w * 2)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001157e6: ble.b $115816
        e.step()?;
        pc = if r.less_equal()? {
            0x115816_u32
        } else {
            0x1157e8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_f4_unit::combat_body_block_f4;

mod combat_body_block_f5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_f5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001157e8: move.w #$1, $438764.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x438764_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001157f0: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 001157f6: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001157fc: move.w (a0, d0.w * 2), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115800: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115804: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0011580a: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0011580c: move.w #$1, $90(a0)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x90_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115812: bra.w $1171d4
        e.step()?;
        pc = 0x1171d4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_f5_unit::combat_body_block_f5;

mod combat_body_block_f6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_f6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115816: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 0011581c: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115822: cmpi.w #$9, (a0, d0.w * 2)
        e.step()?;
        let source = 0x9_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00115828: bge.w $11597c
        e.step()?;
        pc = if r.greater_equal()? {
            0x11597c_u32
        } else {
            0x11582c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_f6_unit::combat_body_block_f6;

mod combat_body_block_f7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_f7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011582c: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 00115832: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115838: cmpi.w #$ffff, (a0, d0.w * 2)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0011583e: ble.w $11597c
        e.step()?;
        pc = if r.less_equal()? {
            0x11597c_u32
        } else {
            0x115842_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_f7_unit::combat_body_block_f7;

mod combat_body_block_f8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_f8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115842: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 00115848: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011584e: lea.l $442742.l, a1
        e.step()?;
        let value = 0x442742_u32;
        r.put_address(1, value);
        // 00115854: move.w (a0, d0.w * 2), d1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00115858: tst.b (a1, d1.w)
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(1));
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0011585c: beq.w $11597c
        e.step()?;
        pc = if r.equal()? {
            0x11597c_u32
        } else {
            0x115860_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_f8_unit::combat_body_block_f8;

mod combat_body_block_f9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_f9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115860: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 00115866: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011586c: move.w (a0, d0.w * 2), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115870: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115874: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0011587a: tst.w $5e(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x5e_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0011587e: bne.w $11597c
        e.step()?;
        pc = if r.not_equal()? {
            0x11597c_u32
        } else {
            0x115882_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_f9_unit::combat_body_block_f9;

mod combat_body_block_fa_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_fa(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115882: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 00115888: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011588e: move.w (a0, d0.w * 2), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115892: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115896: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0011589c: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0011589e: tst.w $92(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x92_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001158a2: bne.w $11597c
        e.step()?;
        pc = if r.not_equal()? {
            0x11597c_u32
        } else {
            0x1158a6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_fa_unit::combat_body_block_fa;

mod combat_body_block_fb_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_fb(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001158a6: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 001158ac: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001158b2: move.w (a0, d0.w * 2), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001158b6: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001158ba: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001158c0: tst.w $50(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x50_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001158c4: ble.w $11597c
        e.step()?;
        pc = if r.less_equal()? {
            0x11597c_u32
        } else {
            0x1158c8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_fb_unit::combat_body_block_fb;

mod combat_body_block_fc_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_fc(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001158c8: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 001158ce: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001158d4: move.w (a0, d0.w * 2), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001158d8: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001158dc: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001158e2: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001158e4: tst.b $9df(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x9df_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001158e8: beq.w $11597c
        e.step()?;
        pc = if r.equal()? {
            0x11597c_u32
        } else {
            0x1158ec_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_fc_unit::combat_body_block_fc;

mod combat_body_block_fd_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_fd(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001158ec: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 001158ee: MacToolbox 0xa974
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa974_u32 as u16))?;
        // 001158f0: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001158f2: beq.b $115952
        e.step()?;
        pc = if r.equal()? {
            0x115952_u32
        } else {
            0x1158f4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_fd_unit::combat_body_block_fd;

mod combat_body_block_fe_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_fe(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001158f4: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 001158fa: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115900: lea.l $442742.l, a1
        e.step()?;
        let value = 0x442742_u32;
        r.put_address(1, value);
        // 00115906: move.w (a0, d0.w * 2), d1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0011590a: clr.b (a1, d1.w)
        e.step()?;
        let value = 0;
        let destination_address = r
            .address(1)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(1));
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 0011590e: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 00115914: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011591a: lea.l $43949c.l, a1
        e.step()?;
        let value = 0x43949c_u32;
        r.put_address(1, value);
        // 00115920: move.w (a0, d0.w * 2), d1
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00115924: movea.l (a1, d1.w * 4), a1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = e.l(value_address)?;
        r.put_address(1, value);
        // 00115928: movea.l (a1), a1
        e.step()?;
        let value_address = r.address(1)?.wrapping_add(0x0_u32);
        let value = e.l(value_address)?;
        r.put_address(1, value);
        // 0011592a: move.l $8(a1), $44292c.l
        e.step()?;
        let value_address = r.address(1)?.wrapping_add(0x8_u32);
        let value = e.l(value_address)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115932: move.l $c(a1), $442930.l
        e.step()?;
        let value_address = r.address(1)?.wrapping_add(0xc_u32);
        let value = e.l(value_address)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011593a: move.w #$8d, -(a7)
        e.step()?;
        let value = 0x8d_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011593e: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115944_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00115944: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115948: bsr.l $1537ee
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11594e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1537ee_u32)?;
        // 0011594e: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00115950: bra.b $11597c
        e.step()?;
        pc = 0x11597c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_fe_unit::combat_body_block_fe;

mod combat_body_block_ff_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_ff(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115952: move.w #$1, $438764.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x438764_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011595a: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 00115960: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115966: move.w (a0, d0.w * 2), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011596a: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011596e: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00115974: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115976: move.w #$1, $90(a0)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0x90_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x11597c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_ff_unit::combat_body_block_ff;

mod combat_body_block_100_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_100(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011597c: subq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 0011597e: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00115980: move.w $447218.l, d0
        e.step()?;
        let value_address = 0x447218_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115986: move.l d0, -(a7)
        e.step()?;
        let value = r.data(0, 32)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115988: pea.l $400.w
        e.step()?;
        let value = 0x400_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0011598c: MacToolbox 0xa858
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa858_u32 as u16))?;
        // 0011598e: move.l (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = e.l(value_address)?;
        r.offset_address(7, 4)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115990: beq.b $11599e
        e.step()?;
        pc = if r.equal()? {
            0x11599e_u32
        } else {
            0x115992_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_100_unit::combat_body_block_100;

mod combat_body_block_101_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_101(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115992: move.w #$15, -(a7)
        e.step()?;
        let value = 0x15_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115996: bsr.l $16474a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11599c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16474a_u32)?;
        // 0011599c: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x11599e_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_101_unit::combat_body_block_101;

mod combat_body_block_102_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_102(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011599e: MacToolbox 0xa9b4
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa9b4_u32 as u16))?;
        // 001159a0: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001159a2: bsr.l $133d46
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1159a8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133d46_u32)?;
        // 001159a8: move.w #$ffff, -(a7)
        e.step()?;
        let value = 0xffff_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001159ac: pea.l $44720a.l
        e.step()?;
        let value = 0x44720a_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001159b2: MacToolbox 0xa970
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa970_u32 as u16))?;
        // 001159b4: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001159b6: moveq #$0, d1
        e.step()?;
        r.put_data(1, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001159b8: move.b d0, d1
        e.step()?;
        let value = r.data(0, 8)?;
        r.put_data(1, 8, value);
        r.move_flags(value, 8);
        // 001159ba: move.w d1, -$1e(a6)
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe2_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001159be: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 001159c0: pea.l $44720a.l
        e.step()?;
        let value = 0x44720a_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001159c6: MacToolbox 0xa97f
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa97f_u32 as u16))?;
        // 001159c8: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001159ca: beq.b $1159fa
        e.step()?;
        pc = if r.equal()? {
            0x1159fa_u32
        } else {
            0x1159cc_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_102_unit::combat_body_block_102;

mod combat_body_block_103_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_103(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001159cc: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 001159ce: pea.l $44720a.l
        e.step()?;
        let value = 0x44720a_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001159d4: pea.l $4394c0.l
        e.step()?;
        let value = 0x4394c0_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001159da: pea.l $4434b2.l
        e.step()?;
        let value = 0x4434b2_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001159e0: MacToolbox 0xa980
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa980_u32 as u16))?;
        // 001159e2: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001159e4: moveq #$0, d1
        e.step()?;
        r.put_data(1, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001159e6: move.b d0, d1
        e.step()?;
        let value = r.data(0, 8)?;
        r.put_data(1, 8, value);
        r.move_flags(value, 8);
        // 001159e8: move.w d1, -$1e(a6)
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe2_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001159ec: cmpi.w #$3, $44720a.l
        e.step()?;
        let source = 0x3_u32;
        let destination_address = 0x44720a_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001159f4: beq.b $115a6c
        e.step()?;
        pc = if r.equal()? {
            0x115a6c_u32
        } else {
            0x1159f6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_103_unit::combat_body_block_103;

mod combat_body_block_104_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_104(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001159f6: bra.w $117338
        e.step()?;
        pc = 0x117338_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_104_unit::combat_body_block_104;

mod combat_body_block_105_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_105(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001159fa: move.w $44720a.l, d0
        e.step()?;
        let value_address = 0x44720a_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115a00: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115a02: beq.w $116b92
        e.step()?;
        pc = if r.equal()? {
            0x116b92_u32
        } else {
            0x115a06_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_105_unit::combat_body_block_105;

mod combat_body_block_106_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_106(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115a06: subq.w #$2, d0
        e.step()?;
        let source = 0x2_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115a08: beq.b $115a6c
        e.step()?;
        pc = if r.equal()? {
            0x115a6c_u32
        } else {
            0x115a0a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_106_unit::combat_body_block_106;

mod combat_body_block_107_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_107(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115a0a: subq.w #$2, d0
        e.step()?;
        let source = 0x2_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115a0c: beq.b $115a6c
        e.step()?;
        pc = if r.equal()? {
            0x115a6c_u32
        } else {
            0x115a0e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_107_unit::combat_body_block_107;

mod combat_body_block_108_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_108(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115a0e: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115a10: beq.b $115a1e
        e.step()?;
        pc = if r.equal()? {
            0x115a1e_u32
        } else {
            0x115a12_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_108_unit::combat_body_block_108;

mod combat_body_block_109_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_109(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115a12: subi.w #$9, d0
        e.step()?;
        let source = 0x9_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115a16: beq.w $1172ba
        e.step()?;
        pc = if r.equal()? {
            0x1172ba_u32
        } else {
            0x115a1a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_109_unit::combat_body_block_109;
mod combat_body_block_10a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_10a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115a1a: bra.w $117338
        e.step()?;
        pc = 0x117338_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_10a_unit::combat_body_block_10a;

mod combat_body_block_10b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_10b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115a1e: move.l $43879c.l, -(a7)
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115a24: MacToolbox 0xa922
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa922_u32 as u16))?;
        // 00115a26: move.l $43879c.l, -(a7)
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115a2c: MacToolbox 0xa923
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa923_u32 as u16))?;
        // 00115a2e: move.l $4386ae.l, -(a7)
        e.step()?;
        let value_address = 0x4386ae_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115a34: MacToolbox 0xa922
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa922_u32 as u16))?;
        // 00115a36: move.l $4386ae.l, -(a7)
        e.step()?;
        let value_address = 0x4386ae_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115a3c: MacToolbox 0xa923
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa923_u32 as u16))?;
        // 00115a3e: move.l $43878c.l, -(a7)
        e.step()?;
        let value_address = 0x43878c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115a44: MacToolbox 0xa922
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa922_u32 as u16))?;
        // 00115a46: move.l $43878c.l, -(a7)
        e.step()?;
        let value_address = 0x43878c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115a4c: MacToolbox 0xa923
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa923_u32 as u16))?;
        // 00115a4e: tst.l $438794.l
        e.step()?;
        let value_address = 0x438794_u32;
        let value = e.l(value_address)?;
        r.move_flags(value, 32);
        // 00115a54: beq.w $117338
        e.step()?;
        pc = if r.equal()? {
            0x117338_u32
        } else {
            0x115a58_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_10b_unit::combat_body_block_10b;

mod combat_body_block_10c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_10c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115a58: move.l $438794.l, -(a7)
        e.step()?;
        let value_address = 0x438794_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115a5e: MacToolbox 0xa922
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa922_u32 as u16))?;
        // 00115a60: move.l $438794.l, -(a7)
        e.step()?;
        let value_address = 0x438794_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115a66: MacToolbox 0xa923
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa923_u32 as u16))?;
        // 00115a68: bra.w $117338
        e.step()?;
        pc = 0x117338_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_10c_unit::combat_body_block_10c;

mod combat_body_block_10d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_10d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115a6c: subq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00115a6e: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00115a70: move.w $447218.l, d0
        e.step()?;
        let value_address = 0x447218_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115a76: move.l d0, -(a7)
        e.step()?;
        let value = r.data(0, 32)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115a78: pea.l $100.w
        e.step()?;
        let value = 0x100_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00115a7c: MacToolbox 0xa858
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa858_u32 as u16))?;
        // 00115a7e: move.l (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = e.l(value_address)?;
        r.offset_address(7, 4)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115a80: beq.b $115ae4
        e.step()?;
        pc = if r.equal()? {
            0x115ae4_u32
        } else {
            0x115a82_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_10d_unit::combat_body_block_10d;

mod combat_body_block_10e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_10e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115a82: subq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00115a84: move.l $44720c.l, -(a7)
        e.step()?;
        let value_address = 0x44720c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115a8a: pea.l $ff.w
        e.step()?;
        let value = 0xff_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00115a8e: MacToolbox 0xa858
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa858_u32 as u16))?;
        // 00115a90: move.l (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = e.l(value_address)?;
        r.offset_address(7, 4)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115a92: move.b d0, -$1f(a6)
        e.step()?;
        let value = r.data(0, 8)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe1_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115a96: subq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00115a98: move.b -$1f(a6), d0
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffe1_u32);
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00115a9c: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115a9e: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115aa0: MacToolbox 0xa93e
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa93e_u32 as u16))?;
        // 00115aa2: move.l (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = e.l(value_address)?;
        r.offset_address(7, 4)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115aa4: move.l d0, $438742.l
        e.step()?;
        let value = r.data(0, 32)?;
        let destination_address = 0x438742_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115aaa: bsr.l $13e13c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115ab0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13e13c_u32)?;
        // 00115ab0: bsr.l $1279bc
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115ab6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1279bc_u32)?;
        // 00115ab6: tst.b $4386b2.l
        e.step()?;
        let value_address = 0x4386b2_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00115abc: beq.b $115ac4
        e.step()?;
        pc = if r.equal()? {
            0x115ac4_u32
        } else {
            0x115abe_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_10e_unit::combat_body_block_10e;

mod combat_body_block_10f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_10f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115abe: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00115ac0: bra.w $117344
        e.step()?;
        pc = 0x117344_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_10f_unit::combat_body_block_10f;

mod combat_body_block_110_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_110(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115ac4: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115aca: add.w d0, d0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115acc: addq.w #$5, d0
        e.step()?;
        let source = 0x5_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115ace: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115ad0: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115ad6: add.w d0, d0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115ad8: addq.w #$5, d0
        e.step()?;
        let source = 0x5_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115ada: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115adc: bsr.l $10e6b6
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115ae2_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x10e6b6_u32)?;
        // 00115ae2: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x115ae4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_110_unit::combat_body_block_110;

mod combat_body_block_111_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_111(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115ae4: subq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00115ae6: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00115ae8: move.w $447218.l, d0
        e.step()?;
        let value_address = 0x447218_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115aee: move.l d0, -(a7)
        e.step()?;
        let value = r.data(0, 32)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115af0: pea.l $800.w
        e.step()?;
        let value = 0x800_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00115af4: MacToolbox 0xa858
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa858_u32 as u16))?;
        // 00115af6: move.l (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = e.l(value_address)?;
        r.offset_address(7, 4)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115af8: beq.b $115b50
        e.step()?;
        pc = if r.equal()? {
            0x115b50_u32
        } else {
            0x115afa_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_111_unit::combat_body_block_111;

mod combat_body_block_112_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_112(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115afa: move.w $44720e.l, $443316.l
        e.step()?;
        let value_address = 0x44720e_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115b04: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115b06: bsr.l $111984
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115b0c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x111984_u32)?;
        // 00115b0c: move.b $442737.l, d0
        e.step()?;
        let value_address = 0x442737_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00115b12: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115b14: add.w d0, d0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115b16: addq.w #$5, d0
        e.step()?;
        let source = 0x5_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115b18: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115b1a: move.b $442736.l, d0
        e.step()?;
        let value_address = 0x442736_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00115b20: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115b22: add.w d0, d0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115b24: addq.w #$5, d0
        e.step()?;
        let source = 0x5_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115b26: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115b28: bsr.l $10e6b6
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115b2e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x10e6b6_u32)?;
        // 00115b2e: clr.b $442737.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442737_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115b34: clr.b $442736.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442736_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115b3a: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00115b3c: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115b3e: move.w d0, $443316.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115b44: clr.l $439404.l
        e.step()?;
        let value = 0;
        let destination_address = 0x439404_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115b4a: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00115b4c: bra.w $117338
        e.step()?;
        pc = 0x117338_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_112_unit::combat_body_block_112;

mod combat_body_block_113_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_113(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115b50: clr.l $439404.l
        e.step()?;
        let value = 0;
        let destination_address = 0x439404_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115b56: move.w $44720e.l, $443316.l
        e.step()?;
        let value_address = 0x44720e_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115b60: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115b62: bsr.l $111984
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115b68_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x111984_u32)?;
        // 00115b68: move.l $43879c.l, -(a7)
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115b6e: MacToolbox 0xa873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 00115b70: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x115b72_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_113_unit::combat_body_block_113;

mod combat_body_block_114_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_114(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115b72: clr.b $437a6b.l
        e.step()?;
        let value = 0;
        let destination_address = 0x437a6b_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115b78: tst.w $438778.l
        e.step()?;
        let value_address = 0x438778_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00115b7e: bne.w $1167fc
        e.step()?;
        pc = if r.not_equal()? {
            0x1167fc_u32
        } else {
            0x115b82_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_114_unit::combat_body_block_114;

mod combat_body_block_115_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_115(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115b82: move.w $443316.l, d0
        e.step()?;
        let value_address = 0x443316_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115b88: subi.w #$61, d0
        e.step()?;
        let source = 0x61_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115b8c: beq.w $115ce8
        e.step()?;
        pc = if r.equal()? {
            0x115ce8_u32
        } else {
            0x115b90_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_115_unit::combat_body_block_115;

mod combat_body_block_116_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_116(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115b90: subi.w #$112, d0
        e.step()?;
        let source = 0x112_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115b94: beq.w $115d04
        e.step()?;
        pc = if r.equal()? {
            0x115d04_u32
        } else {
            0x115b98_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_116_unit::combat_body_block_116;

mod combat_body_block_117_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_117(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115b98: subi.w #$f1, d0
        e.step()?;
        let source = 0xf1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115b9c: beq.w $116384
        e.step()?;
        pc = if r.equal()? {
            0x116384_u32
        } else {
            0x115ba0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_117_unit::combat_body_block_117;

mod combat_body_block_118_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_118(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115ba0: subi.w #$102, d0
        e.step()?;
        let source = 0x102_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115ba4: beq.w $116710
        e.step()?;
        pc = if r.equal()? {
            0x116710_u32
        } else {
            0x115ba8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_118_unit::combat_body_block_118;

mod combat_body_block_119_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_119(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115ba8: subi.w #$201, d0
        e.step()?;
        let source = 0x201_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115bac: beq.w $116166
        e.step()?;
        pc = if r.equal()? {
            0x116166_u32
        } else {
            0x115bb0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_119_unit::combat_body_block_119;
mod combat_body_block_11a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_11a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115bb0: subi.w #$2fc, d0
        e.step()?;
        let source = 0x2fc_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115bb4: beq.w $116674
        e.step()?;
        pc = if r.equal()? {
            0x116674_u32
        } else {
            0x115bb8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_11a_unit::combat_body_block_11a;

mod combat_body_block_11b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_11b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115bb8: subi.w #$2ff, d0
        e.step()?;
        let source = 0x2ff_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115bbc: beq.w $1161e4
        e.step()?;
        pc = if r.equal()? {
            0x1161e4_u32
        } else {
            0x115bc0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_11b_unit::combat_body_block_11b;

mod combat_body_block_11c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_11c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115bc0: subi.w #$215, d0
        e.step()?;
        let source = 0x215_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115bc4: beq.w $116794
        e.step()?;
        pc = if r.equal()? {
            0x116794_u32
        } else {
            0x115bc8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_11c_unit::combat_body_block_11c;

mod combat_body_block_11d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_11d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115bc8: subi.w #$ee, d0
        e.step()?;
        let source = 0xee_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115bcc: beq.w $115f28
        e.step()?;
        pc = if r.equal()? {
            0x115f28_u32
        } else {
            0x115bd0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_11d_unit::combat_body_block_11d;

mod combat_body_block_11e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_11e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115bd0: subi.w #$10d, d0
        e.step()?;
        let source = 0x10d_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115bd4: beq.w $1162fe
        e.step()?;
        pc = if r.equal()? {
            0x1162fe_u32
        } else {
            0x115bd8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_11e_unit::combat_body_block_11e;

mod combat_body_block_11f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_11f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115bd8: subi.w #$202, d0
        e.step()?;
        let source = 0x202_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115bdc: beq.w $1167f2
        e.step()?;
        pc = if r.equal()? {
            0x1167f2_u32
        } else {
            0x115be0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_11f_unit::combat_body_block_11f;

mod combat_body_block_120_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_120(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115be0: subi.w #$bd, d0
        e.step()?;
        let source = 0xbd_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115be4: beq.b $115c62
        e.step()?;
        pc = if r.equal()? {
            0x115c62_u32
        } else {
            0x115be6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_120_unit::combat_body_block_120;

mod combat_body_block_121_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_121(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115be6: subi.w #$101, d0
        e.step()?;
        let source = 0x101_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115bea: beq.w $115c6e
        e.step()?;
        pc = if r.equal()? {
            0x115c6e_u32
        } else {
            0x115bee_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_121_unit::combat_body_block_121;

mod combat_body_block_122_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_122(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115bee: subi.w #$101, d0
        e.step()?;
        let source = 0x101_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115bf2: beq.w $115c7a
        e.step()?;
        pc = if r.equal()? {
            0x115c7a_u32
        } else {
            0x115bf6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_122_unit::combat_body_block_122;

mod combat_body_block_123_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_123(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115bf6: subi.w #$101, d0
        e.step()?;
        let source = 0x101_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115bfa: beq.w $115c86
        e.step()?;
        pc = if r.equal()? {
            0x115c86_u32
        } else {
            0x115bfe_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_123_unit::combat_body_block_123;

mod combat_body_block_124_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_124(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115bfe: subi.w #$102, d0
        e.step()?;
        let source = 0x102_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115c02: beq.w $115c9e
        e.step()?;
        pc = if r.equal()? {
            0x115c9e_u32
        } else {
            0x115c06_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_124_unit::combat_body_block_124;

mod combat_body_block_125_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_125(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c06: subi.w #$ff, d0
        e.step()?;
        let source = 0xff_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115c0a: beq.w $115c92
        e.step()?;
        pc = if r.equal()? {
            0x115c92_u32
        } else {
            0x115c0e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_125_unit::combat_body_block_125;

mod combat_body_block_126_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_126(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c0e: subi.w #$204, d0
        e.step()?;
        let source = 0x204_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115c12: beq.w $115cc2
        e.step()?;
        pc = if r.equal()? {
            0x115cc2_u32
        } else {
            0x115c16_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_126_unit::combat_body_block_126;

mod combat_body_block_127_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_127(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c16: subi.w #$fe, d0
        e.step()?;
        let source = 0xfe_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115c1a: beq.w $115caa
        e.step()?;
        pc = if r.equal()? {
            0x115caa_u32
        } else {
            0x115c1e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_127_unit::combat_body_block_127;

mod combat_body_block_128_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_128(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c1e: subi.w #$201, d0
        e.step()?;
        let source = 0x201_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115c22: beq.w $115cb6
        e.step()?;
        pc = if r.equal()? {
            0x115cb6_u32
        } else {
            0x115c26_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_128_unit::combat_body_block_128;

mod combat_body_block_129_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_129(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c26: subi.w #$f8, d0
        e.step()?;
        let source = 0xf8_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115c2a: beq.w $115cce
        e.step()?;
        pc = if r.equal()? {
            0x115cce_u32
        } else {
            0x115c2e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_129_unit::combat_body_block_129;
mod combat_body_block_12a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_12a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c2e: subi.w #$345, d0
        e.step()?;
        let source = 0x345_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115c32: beq.w $115d12
        e.step()?;
        pc = if r.equal()? {
            0x115d12_u32
        } else {
            0x115c36_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_12a_unit::combat_body_block_12a;

mod combat_body_block_12b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_12b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c36: subi.w #$1f4, d0
        e.step()?;
        let source = 0x1f4_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115c3a: beq.w $115cda
        e.step()?;
        pc = if r.equal()? {
            0x115cda_u32
        } else {
            0x115c3e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_12b_unit::combat_body_block_12b;

mod combat_body_block_12c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_12c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c3e: subi.w #$107, d0
        e.step()?;
        let source = 0x107_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115c42: beq.w $1165ea
        e.step()?;
        pc = if r.equal()? {
            0x1165ea_u32
        } else {
            0x115c46_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_12c_unit::combat_body_block_12c;

mod combat_body_block_12d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_12d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c46: subi.w #$1fc, d0
        e.step()?;
        let source = 0x1fc_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115c4a: beq.w $115cf6
        e.step()?;
        pc = if r.equal()? {
            0x115cf6_u32
        } else {
            0x115c4e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_12d_unit::combat_body_block_12d;

mod combat_body_block_12e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_12e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c4e: subi.w #$802, d0
        e.step()?;
        let source = 0x802_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115c52: beq.w $116560
        e.step()?;
        pc = if r.equal()? {
            0x116560_u32
        } else {
            0x115c56_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_12e_unit::combat_body_block_12e;

mod combat_body_block_12f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_12f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c56: subi.w #$ff, d0
        e.step()?;
        let source = 0xff_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115c5a: beq.w $11651e
        e.step()?;
        pc = if r.equal()? {
            0x11651e_u32
        } else {
            0x115c5e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_12f_unit::combat_body_block_12f;

mod combat_body_block_130_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_130(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c5e: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_130_unit::combat_body_block_130;

mod combat_body_block_131_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_131(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c62: move.b #$1, $437a6b.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x437a6b_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115c6a: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_131_unit::combat_body_block_131;

mod combat_body_block_132_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_132(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c6e: move.b #$2, $437a6b.l
        e.step()?;
        let value = 0x2_u32;
        let destination_address = 0x437a6b_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115c76: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_132_unit::combat_body_block_132;

mod combat_body_block_133_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_133(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c7a: move.b #$3, $437a6b.l
        e.step()?;
        let value = 0x3_u32;
        let destination_address = 0x437a6b_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115c82: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_133_unit::combat_body_block_133;

mod combat_body_block_134_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_134(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c86: move.b #$4, $437a6b.l
        e.step()?;
        let value = 0x4_u32;
        let destination_address = 0x437a6b_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115c8e: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_134_unit::combat_body_block_134;

mod combat_body_block_135_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_135(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c92: move.b #$5, $437a6b.l
        e.step()?;
        let value = 0x5_u32;
        let destination_address = 0x437a6b_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115c9a: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_135_unit::combat_body_block_135;

mod combat_body_block_136_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_136(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115c9e: move.b #$6, $437a6b.l
        e.step()?;
        let value = 0x6_u32;
        let destination_address = 0x437a6b_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115ca6: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_136_unit::combat_body_block_136;

mod combat_body_block_137_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_137(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115caa: move.b #$7, $437a6b.l
        e.step()?;
        let value = 0x7_u32;
        let destination_address = 0x437a6b_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115cb2: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_137_unit::combat_body_block_137;

mod combat_body_block_138_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_138(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115cb6: move.b #$8, $437a6b.l
        e.step()?;
        let value = 0x8_u32;
        let destination_address = 0x437a6b_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115cbe: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_138_unit::combat_body_block_138;

mod combat_body_block_139_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_139(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115cc2: move.b #$9, $437a6b.l
        e.step()?;
        let value = 0x9_u32;
        let destination_address = 0x437a6b_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115cca: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_139_unit::combat_body_block_139;
mod combat_body_block_13a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_13a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115cce: move.b #$a, $437a6b.l
        e.step()?;
        let value = 0xa_u32;
        let destination_address = 0x437a6b_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00115cd6: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_13a_unit::combat_body_block_13a;

mod combat_body_block_13b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_13b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115cda: move.l $43945c.l, $439404.l
        e.step()?;
        let value_address = 0x43945c_u32;
        let value = e.l(value_address)?;
        let destination_address = 0x439404_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115ce4: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_13b_unit::combat_body_block_13b;

mod combat_body_block_13c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_13c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115ce8: move.l $439458.l, $439404.l
        e.step()?;
        let value_address = 0x439458_u32;
        let value = e.l(value_address)?;
        let destination_address = 0x439404_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115cf2: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_13c_unit::combat_body_block_13c;

mod combat_body_block_13d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_13d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115cf6: move.l $439474.l, $439404.l
        e.step()?;
        let value_address = 0x439474_u32;
        let value = e.l(value_address)?;
        let destination_address = 0x439404_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115d00: bra.w $1171d4
        e.step()?;
        pc = 0x1171d4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_13d_unit::combat_body_block_13d;

mod combat_body_block_13e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_13e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115d04: move.l $439470.l, $439404.l
        e.step()?;
        let value_address = 0x439470_u32;
        let value = e.l(value_address)?;
        let destination_address = 0x439404_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115d0e: bra.w $1171d4
        e.step()?;
        pc = 0x1171d4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_13e_unit::combat_body_block_13e;

mod combat_body_block_13f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_13f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115d12: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115d18: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 00115d1a: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00115d20: addi.w #$170, d1
        e.step()?;
        let source = 0x170_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00115d24: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00115d26: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115d2c: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 00115d2e: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115d34: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115d3a: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115d40: addi.w #$1a8, d0
        e.step()?;
        let source = 0x1a8_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115d44: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115d4a: moveq #$29, d0
        e.step()?;
        r.put_data(0, 32, 0x29_u32);
        r.move_flags(0x29_u32, 32);
        // 00115d4c: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115d52: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115d58: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115d5c: bsr.l $153720
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115d62_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153720_u32)?;
        // 00115d62: tst.w $438786.l
        e.step()?;
        let value_address = 0x438786_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00115d68: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00115d6a: beq.w $115f0e
        e.step()?;
        pc = if r.equal()? {
            0x115f0e_u32
        } else {
            0x115d6e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_13f_unit::combat_body_block_13f;

mod combat_body_block_140_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_140(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115d6e: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115d74: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115d78: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00115d7e: move.w $4435d2.l, d1
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00115d84: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00115d88: lea.l $448aa0.l, a1
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(1, value);
        // 00115d8e: move.w $12(a1, d1.l), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x12_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00115d92: add.w $1e(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x1e_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00115d96: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115d9c: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115da0: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00115da6: cmp.w $40(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x40_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00115daa: bne.w $115f0e
        e.step()?;
        pc = if r.not_equal()? {
            0x115f0e_u32
        } else {
            0x115dae_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_140_unit::combat_body_block_140;

mod combat_body_block_141_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_141(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115dae: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115db4: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115db8: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00115dbe: tst.w $60(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x60_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00115dc2: bne.w $1167fc
        e.step()?;
        pc = if r.not_equal()? {
            0x1167fc_u32
        } else {
            0x115dc6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_141_unit::combat_body_block_141;

mod combat_body_block_142_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_142(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115dc6: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115dcc: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115dd0: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00115dd6: tst.w $10(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00115dda: bne.w $1167fc
        e.step()?;
        pc = if r.not_equal()? {
            0x1167fc_u32
        } else {
            0x115dde_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_142_unit::combat_body_block_142;

mod combat_body_block_143_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_143(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115dde: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115de4: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115de8: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00115dee: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115df0: tst.w $98(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x98_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00115df4: bne.w $1167fc
        e.step()?;
        pc = if r.not_equal()? {
            0x1167fc_u32
        } else {
            0x115df8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_143_unit::combat_body_block_143;

mod combat_body_block_144_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_144(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115df8: move.w #$298, -(a7)
        e.step()?;
        let value = 0x298_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115dfc: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115e02_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00115e02: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115e08: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115e0c: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00115e12: move.w $4435d2.l, d1
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00115e18: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 00115e1c: lea.l $448aa0.l, a1
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(1, value);
        // 00115e22: move.w $12(a1, d1.l), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x12_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00115e26: add.w $1e(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x1e_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00115e2a: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115e30: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115e34: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00115e3a: lea.l $40(a0, d0.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x40_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 00115e3e: sub.w d1, (a0)
        e.step()?;
        let source = r.data(1, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00115e40: move.w $4435d2.l, -(a7)
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115e46: bsr.l $120c14
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115e4c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x120c14_u32)?;
        // 00115e4c: move.w $4435d2.l, -(a7)
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115e52: bsr.l $105366
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115e58_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x105366_u32)?;
        // 00115e58: move.b $438740.l, d0
        e.step()?;
        let value_address = 0x438740_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00115e5e: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115e60: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115e62: sub.l $442780.l, d0
        e.step()?;
        let source_address = 0x442780_u32;
        let source = e.l(source_address)?;
        let destination = r.data(0, 32)?;
        let value = r.subtract_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00115e68: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 00115e6e: move.w $4435d2.l, d1
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00115e74: move.w d0, (a0, d1.w * 4)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(4));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115e78: move.b $438741.l, d0
        e.step()?;
        let value_address = 0x438741_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00115e7e: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115e80: ext.l d0
        e.step()?;
        let source = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115e82: sub.l $442784.l, d0
        e.step()?;
        let source_address = 0x442784_u32;
        let source = e.l(source_address)?;
        let destination = r.data(0, 32)?;
        let value = r.subtract_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 00115e88: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 00115e8e: move.w $4435d2.l, d1
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00115e94: move.w d0, $2(a0, d1.w * 4)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(4));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115e98: move.b $438740.l, d0
        e.step()?;
        let value_address = 0x438740_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00115e9e: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115ea0: mulu.w #$b4, d0
        e.step()?;
        let source = 0xb4_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115ea4: lea.l $4085b8.l, a0
        e.step()?;
        let value = 0x4085b8_u32;
        r.put_address(0, value);
        // 00115eaa: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115eac: move.b $438741.l, d0
        e.step()?;
        let value_address = 0x438741_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00115eb2: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115eb4: lea.l $437a6e.l, a1
        e.step()?;
        let value = 0x437a6e_u32;
        r.put_address(1, value);
        // 00115eba: move.w $4435d2.l, d1
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00115ec0: move.w (a0, d0.l * 2), (a1, d1.w * 2)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r
            .address(1)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115ec6: move.b $438740.l, d0
        e.step()?;
        let value_address = 0x438740_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00115ecc: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115ece: mulu.w #$b4, d0
        e.step()?;
        let source = 0xb4_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115ed2: lea.l $4085b8.l, a0
        e.step()?;
        let value = 0x4085b8_u32;
        r.put_address(0, value);
        // 00115ed8: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115eda: move.b $438741.l, d0
        e.step()?;
        let value_address = 0x438741_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00115ee0: extb.l d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115ee2: move.w $4435d2.l, (a0, d0.l * 2)
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115eea: move.l $43879c.l, -(a7)
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115ef0: MacToolbox 0xa873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 00115ef2: subq.w #$1, $447194.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x447194_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 00115ef8: move.l $43878c.l, -(a7)
        e.step()?;
        let value_address = 0x43878c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115efe: MacToolbox 0xa873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 00115f00: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115f02: bsr.l $125a3a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115f08_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x125a3a_u32)?;
        // 00115f08: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00115f0a: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_144_unit::combat_body_block_144;

mod combat_body_block_145_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_145(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115f0e: move.w #$52, -(a7)
        e.step()?;
        let value = 0x52_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115f12: bsr.l $16474a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115f18_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16474a_u32)?;
        // 00115f18: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115f1c: bsr.l $1537ee
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115f22_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1537ee_u32)?;
        // 00115f22: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00115f24: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_145_unit::combat_body_block_145;

mod combat_body_block_146_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_146(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115f28: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115f2e: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 00115f30: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00115f36: addi.w #$170, d1
        e.step()?;
        let source = 0x170_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00115f3a: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00115f3c: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115f42: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 00115f44: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115f4a: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115f50: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115f56: addi.w #$16e, d0
        e.step()?;
        let source = 0x16e_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115f5a: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115f60: moveq #$33, d0
        e.step()?;
        r.put_data(0, 32, 0x33_u32);
        r.move_flags(0x33_u32, 32);
        // 00115f62: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00115f68: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115f6e: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115f72: bsr.l $153720
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115f78_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153720_u32)?;
        // 00115f78: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00115f7a: move.w $4435d2.l, -(a7)
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115f80: bsr.l $125538
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115f86_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x125538_u32)?;
        // 00115f86: cmpi.w #$a, $4436b4.l
        e.step()?;
        let source = 0xa_u32;
        let destination_address = 0x4436b4_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00115f8e: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00115f90: bge.b $115fac
        e.step()?;
        pc = if r.greater_equal()? {
            0x115fac_u32
        } else {
            0x115f92_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_146_unit::combat_body_block_146;

mod combat_body_block_147_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_147(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115f92: move.w #$51, -(a7)
        e.step()?;
        let value = 0x51_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115f96: bsr.l $16474a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115f9c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16474a_u32)?;
        // 00115f9c: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00115fa0: bsr.l $1537ee
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x115fa6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1537ee_u32)?;
        // 00115fa6: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00115fa8: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_147_unit::combat_body_block_147;

mod combat_body_block_148_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_148(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115fac: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115fb2: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115fb6: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00115fbc: tst.w $60(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x60_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00115fc0: bne.b $11601c
        e.step()?;
        pc = if r.not_equal()? {
            0x11601c_u32
        } else {
            0x115fc2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_148_unit::combat_body_block_148;

mod combat_body_block_149_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_149(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115fc2: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115fc8: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115fcc: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00115fd2: tst.w $10(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00115fd6: bne.b $11601c
        e.step()?;
        pc = if r.not_equal()? {
            0x11601c_u32
        } else {
            0x115fd8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_149_unit::combat_body_block_149;
mod combat_body_block_14a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_14a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115fd8: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115fde: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115fe2: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00115fe8: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00115fea: tst.w $98(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x98_u32);
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00115fee: bne.b $11601c
        e.step()?;
        pc = if r.not_equal()? {
            0x11601c_u32
        } else {
            0x115ff0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_14a_unit::combat_body_block_14a;

mod combat_body_block_14b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_14b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00115ff0: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00115ff6: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00115ffa: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00116000: tst.w $62(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x62_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00116004: bne.b $11601c
        e.step()?;
        pc = if r.not_equal()? {
            0x11601c_u32
        } else {
            0x116006_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_14b_unit::combat_body_block_14b;

mod combat_body_block_14c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_14c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116006: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011600c: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00116010: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00116016: tst.w $6a(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x6a_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0011601a: beq.b $116036
        e.step()?;
        pc = if r.equal()? {
            0x116036_u32
        } else {
            0x11601c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_14c_unit::combat_body_block_14c;

mod combat_body_block_14d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_14d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011601c: move.w #$53, -(a7)
        e.step()?;
        let value = 0x53_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116020: bsr.l $16474a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116026_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16474a_u32)?;
        // 00116026: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011602a: bsr.l $1537ee
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116030_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1537ee_u32)?;
        // 00116030: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00116032: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_14d_unit::combat_body_block_14d;

mod combat_body_block_14e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_14e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116036: pea.l $403df4.l
        e.step()?;
        let value = 0x403df4_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0011603c: pea.l $403e04.l
        e.step()?;
        let value = 0x403e04_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116042: bsr.l $14d596
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116048_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x14d596_u32)?;
        // 00116048: subq.w #$2, d0
        e.step()?;
        let source = 0x2_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011604a: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0011604c: bne.w $1167fc
        e.step()?;
        pc = if r.not_equal()? {
            0x1167fc_u32
        } else {
            0x116050_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_14e_unit::combat_body_block_14e;

mod combat_body_block_14f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_14f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116050: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116052: move.w $4435d2.l, -(a7)
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116058: bsr.l $105572
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11605e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x105572_u32)?;
        // 0011605e: move.w $4435d2.l, -(a7)
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116064: bsr.l $105366
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11606a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x105366_u32)?;
        // 0011606a: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 00116070: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116076: move.w #$ffff, $2(a0, d0.w * 4)
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011607c: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 00116082: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116088: move.w #$ffff, (a0, d0.w * 4)
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011608e: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116094: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00116098: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0011609e: move.w #$ffff, $20(a0, d0.l)
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x20_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001160a4: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 001160aa: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001160b0: move.w #$ffff, (a0, d0.w * 2)
        e.step()?;
        let value = 0xffff_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001160b6: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001160bc: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001160c0: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001160c6: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001160c8: clr.b $9df(a0)
        e.step()?;
        let value = 0;
        let destination_address = r.address(0)?.wrapping_add(0x9df_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001160cc: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001160d2: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001160d6: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001160dc: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001160de: addi.l #$c8, $a72(a0)
        e.step()?;
        let source = 0xc8_u32;
        let destination_address = r.address(0)?.wrapping_add(0xa72_u32);
        let destination = e.l(destination_address)?;
        let value = r.add_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 001160e6: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001160e8: move.w $4435d2.l, -(a7)
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001160ee: bsr.l $160fb2
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1160f4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x160fb2_u32)?;
        // 001160f4: clr.w $4471ea.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4471ea_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001160fa: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001160fc: lea.l $a(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xa_u32);
        r.put_address(7, value);
        // 00116100: bra.b $116132
        e.step()?;
        pc = 0x116132_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_14f_unit::combat_body_block_14f;

mod combat_body_block_150_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_150(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116102: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116104: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00116108: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0011610e: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00116110: tst.b $9df(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x9df_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00116114: beq.b $116130
        e.step()?;
        pc = if r.equal()? {
            0x116130_u32
        } else {
            0x116116_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_150_unit::combat_body_block_150;

mod combat_body_block_151_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_151(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116116: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116118: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011611c: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00116122: tst.w $10(a0, d0.l)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x10_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00116126: bne.b $116130
        e.step()?;
        pc = if r.not_equal()? {
            0x116130_u32
        } else {
            0x116128_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_151_unit::combat_body_block_151;

mod combat_body_block_152_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_152(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116128: move.w #$1, $4471ea.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x4471ea_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x116130_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_152_unit::combat_body_block_152;

mod combat_body_block_153_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_153(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116130: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x116132_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_153_unit::combat_body_block_153;

mod combat_body_block_154_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_154(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116132: move.b $44273a.l, d0
        e.step()?;
        let value_address = 0x44273a_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116138: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011613a: cmp.w d0, d3
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0011613c: ble.b $116102
        e.step()?;
        pc = if r.less_equal()? {
            0x116102_u32
        } else {
            0x11613e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_154_unit::combat_body_block_154;

mod combat_body_block_155_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_155(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011613e: tst.w $4471ea.l
        e.step()?;
        let value_address = 0x4471ea_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00116144: bne.b $116158
        e.step()?;
        pc = if r.not_equal()? {
            0x116158_u32
        } else {
            0x116146_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_155_unit::combat_body_block_155;

mod combat_body_block_156_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_156(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116146: move.w $4435c4.l, $44374c.l
        e.step()?;
        let value_address = 0x4435c4_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x44374c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116150: move.b #$1, $4386fc.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x4386fc_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x116158_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_156_unit::combat_body_block_156;

mod combat_body_block_157_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_157(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116158: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011615a: bsr.l $125a3a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116160_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x125a3a_u32)?;
        // 00116160: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00116162: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_157_unit::combat_body_block_157;

mod combat_body_block_158_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_158(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116166: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011616c: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0011616e: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00116174: addi.w #$148, d1
        e.step()?;
        let source = 0x148_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116178: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0011617a: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116180: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 00116182: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116188: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011618e: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116194: addi.w #$16e, d0
        e.step()?;
        let source = 0x16e_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116198: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011619e: moveq #$2e, d0
        e.step()?;
        r.put_data(0, 32, 0x2e_u32);
        r.move_flags(0x2e_u32, 32);
        // 001161a0: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001161a6: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001161ac: move.w #$2789, -(a7)
        e.step()?;
        let value = 0x2789_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001161b0: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1161b6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 001161b6: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001161ba: bsr.l $153720
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1161c0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153720_u32)?;
        // 001161c0: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001161c6: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001161ca: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001161d0: move.w #$1, $16(a0, d0.l)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x16_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001161d6: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001161d8: bsr.l $125a3a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1161de_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x125a3a_u32)?;
        // 001161de: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001161e0: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_158_unit::combat_body_block_158;

mod combat_body_block_159_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_159(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001161e4: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001161ea: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 001161ec: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001161f2: addi.w #$184, d1
        e.step()?;
        let source = 0x184_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 001161f6: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 001161f8: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001161fe: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 00116200: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116206: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011620c: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116212: addi.w #$16e, d0
        e.step()?;
        let source = 0x16e_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116216: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011621c: moveq #$39, d0
        e.step()?;
        r.put_data(0, 32, 0x39_u32);
        r.move_flags(0x39_u32, 32);
        // 0011621e: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116224: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011622a: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011622e: bsr.l $153720
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116234_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153720_u32)?;
        // 00116234: tst.w $438786.l
        e.step()?;
        let value_address = 0x438786_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0011623a: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0011623c: bne.b $116258
        e.step()?;
        pc = if r.not_equal()? {
            0x116258_u32
        } else {
            0x11623e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_159_unit::combat_body_block_159;
mod combat_body_block_15a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_15a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011623e: move.w #$54, -(a7)
        e.step()?;
        let value = 0x54_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116242: bsr.l $16474a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116248_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16474a_u32)?;
        // 00116248: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011624c: bsr.l $1537ee
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116252_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1537ee_u32)?;
        // 00116252: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00116254: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_15a_unit::combat_body_block_15a;

mod combat_body_block_15b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_15b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116258: move.l #$ffff2779, -(a7)
        e.step()?;
        let value = 0xffff2779_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011625e: move.l #$1e0064, -(a7)
        e.step()?;
        let value = 0x1e0064_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116264: pea.l $403dd6.l
        e.step()?;
        let value = 0x403dd6_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0011626a: bsr.l $123d1c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116270_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x123d1c_u32)?;
        // 00116270: move.l $43878c.l, -(a7)
        e.step()?;
        let value_address = 0x43878c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116276: MacToolbox 0xa873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 00116278: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011627c: clr.l -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011627e: bsr.l $12512a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116284_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x12512a_u32)?;
        // 00116284: move.l $43879c.l, -(a7)
        e.step()?;
        let value_address = 0x43879c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011628a: MacToolbox 0xa873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 0011628c: move.l #$ffff0000, -(a7)
        e.step()?;
        let value = 0xffff0000_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116292: move.l #$1e0064, -(a7)
        e.step()?;
        let value = 0x1e0064_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116298: pea.l $403dd5.l
        e.step()?;
        let value = 0x403dd5_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0011629e: bsr.l $123d1c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1162a4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x123d1c_u32)?;
        // 001162a4: move.l $43878c.l, -(a7)
        e.step()?;
        let value_address = 0x43878c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001162aa: MacToolbox 0xa873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 001162ac: moveq #$0, d3
        e.step()?;
        r.put_data(3, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001162ae: lea.l $1e(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0x1e_u32);
        r.put_address(7, value);
        // 001162b2: bra.b $1162e4
        e.step()?;
        pc = 0x1162e4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_15b_unit::combat_body_block_15b;

mod combat_body_block_15c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_15c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001162b4: lea.l $4434e0.l, a0
        e.step()?;
        let value = 0x4434e0_u32;
        r.put_address(0, value);
        // 001162ba: tst.w (a0, d3.w * 2)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(3, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001162be: beq.b $1162e2
        e.step()?;
        pc = if r.equal()? {
            0x1162e2_u32
        } else {
            0x1162c0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_15c_unit::combat_body_block_15c;

mod combat_body_block_15d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_15d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001162c0: move.w d3, d0
        e.step()?;
        let value = r.data(3, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001162c2: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001162c6: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001162cc: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001162ce: clr.b $9de(a0)
        e.step()?;
        let value = 0;
        let destination_address = r.address(0)?.wrapping_add(0x9de_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001162d2: move.l #$10000, -(a7)
        e.step()?;
        let value = 0x10000_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001162d8: move.w d3, -(a7)
        e.step()?;
        let value = r.data(3, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001162da: bsr.l $161b80
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1162e0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x161b80_u32)?;
        // 001162e0: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1162e2_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_15d_unit::combat_body_block_15d;

mod combat_body_block_15e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_15e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001162e2: addq.w #$1, d3
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(3, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(3, 16, value);
        pc = 0x1162e4_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_15e_unit::combat_body_block_15e;

mod combat_body_block_15f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_15f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001162e4: move.b $44273a.l, d0
        e.step()?;
        let value_address = 0x44273a_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001162ea: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001162ec: cmp.w d0, d3
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(3, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001162ee: ble.b $1162b4
        e.step()?;
        pc = if r.less_equal()? {
            0x1162b4_u32
        } else {
            0x1162f0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_15f_unit::combat_body_block_15f;

mod combat_body_block_160_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_160(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001162f0: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001162f2: bsr.l $125a3a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1162f8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x125a3a_u32)?;
        // 001162f8: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001162fa: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_160_unit::combat_body_block_160;

mod combat_body_block_161_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_161(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001162fe: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116304: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 00116306: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0011630c: addi.w #$15c, d1
        e.step()?;
        let source = 0x15c_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116310: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116312: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116318: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 0011631a: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116320: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116326: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011632c: addi.w #$16e, d0
        e.step()?;
        let source = 0x16e_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116330: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116336: moveq #$63, d0
        e.step()?;
        r.put_data(0, 32, 0x63_u32);
        r.move_flags(0x63_u32, 32);
        // 00116338: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011633e: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116344: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116348: bsr.l $153720
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11634e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153720_u32)?;
        // 0011634e: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116350: bsr.l $1580be
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116356_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1580be_u32)?;
        // 00116356: move.w $408572.l, -(a7)
        e.step()?;
        let value_address = 0x408572_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011635c: move.w $408570.l, -(a7)
        e.step()?;
        let value_address = 0x408570_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116362: bsr.l $10e6b6
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116368_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x10e6b6_u32)?;
        // 00116368: move.l #$ffff, -(a7)
        e.step()?;
        let value = 0xffff_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011636e: move.l (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = e.l(value_address)?;
        r.offset_address(7, 4)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00116370: MacToolbox 0xa032
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa032_u32 as u16))?;
        // 00116372: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116376: bsr.l $1537ee
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11637c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1537ee_u32)?;
        // 0011637c: lea.l $a(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xa_u32);
        r.put_address(7, value);
        // 00116380: bra.w $11597c
        e.step()?;
        pc = 0x11597c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_161_unit::combat_body_block_161;

mod combat_body_block_162_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_162(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116384: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011638a: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011638e: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00116394: move.w $4435d2.l, d1
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0011639a: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 0011639e: lea.l $448aa0.l, a1
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(1, value);
        // 001163a4: move.w $3c(a1, d1.l), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x3c_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001163a8: cmp.w $3e(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x3e_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 001163ac: bne.w $116506
        e.step()?;
        pc = if r.not_equal()? {
            0x116506_u32
        } else {
            0x1163b0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_162_unit::combat_body_block_162;

mod combat_body_block_163_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_163(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001163b0: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001163b6: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001163ba: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001163c0: move.w $4435d2.l, d1
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001163c6: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 001163ca: lea.l $448aa0.l, a1
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(1, value);
        // 001163d0: move.w $12(a1, d1.l), d1
        e.step()?;
        let value_address = r
            .address(1)?
            .wrapping_add(0x12_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001163d4: add.w $1e(a0, d0.l), d1
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x1e_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 001163d8: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001163de: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001163e2: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001163e8: lea.l $40(a0, d0.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x40_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 001163ec: sub.w d1, (a0)
        e.step()?;
        let source = r.data(1, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001163ee: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001163f4: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 001163f6: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001163fc: addi.w #$148, d1
        e.step()?;
        let source = 0x148_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116400: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116402: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116408: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 0011640a: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116410: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116416: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011641c: addi.w #$1a3, d0
        e.step()?;
        let source = 0x1a3_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116420: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116426: moveq #$2e, d0
        e.step()?;
        r.put_data(0, 32, 0x2e_u32);
        r.move_flags(0x2e_u32, 32);
        // 00116428: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011642e: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116434: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116438: bsr.l $153720
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11643e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153720_u32)?;
        // 0011643e: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 00116444: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011644a: move.w (a0, d0.w * 2), d4
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(4, 16, value);
        r.move_flags(value, 16);
        // 0011644e: move.w $447194.l, $4434ae.l
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4434ae_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116458: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0011645a: bra.b $1164ce
        e.step()?;
        pc = 0x1164ce_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_163_unit::combat_body_block_163;

mod combat_body_block_164_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_164(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011645c: move.w $4434ae.l, d0
        e.step()?;
        let value_address = 0x4434ae_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116462: addq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116464: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 0011646a: lea.l $4470b8.l, a1
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(1, value);
        // 00116470: move.w $4434ae.l, d1
        e.step()?;
        let value_address = 0x4434ae_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00116476: move.w (a0, d0.w * 2), (a1, d1.w * 2)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r
            .address(1)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011647c: move.b $44273a.l, d0
        e.step()?;
        let value_address = 0x44273a_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116482: ext.w d0
        e.step()?;
        let source = r.data(0, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116484: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 0011648a: move.w $4434ae.l, d1
        e.step()?;
        let value_address = 0x4434ae_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00116490: cmp.w (a0, d1.w * 2), d0
        e.step()?;
        let source_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00116494: blt.b $1164c8
        e.step()?;
        pc = if r.less()? {
            0x1164c8_u32
        } else {
            0x116496_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_164_unit::combat_body_block_164;

mod combat_body_block_165_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_165(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116496: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 0011649c: move.w $4434ae.l, d0
        e.step()?;
        let value_address = 0x4434ae_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001164a2: cmpi.w #$ffff, (a0, d0.w * 2)
        e.step()?;
        let source = 0xffff_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001164a8: ble.b $1164c8
        e.step()?;
        pc = if r.less_equal()? {
            0x1164c8_u32
        } else {
            0x1164aa_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_165_unit::combat_body_block_165;

mod combat_body_block_166_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_166(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001164aa: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 001164b0: move.w $4434ae.l, d0
        e.step()?;
        let value_address = 0x4434ae_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001164b6: move.w (a0, d0.w * 2), d0
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001164ba: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001164be: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001164c4: subq.w #$1, $20(a0, d0.l)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x20_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x1164c8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_166_unit::combat_body_block_166;

mod combat_body_block_167_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_167(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001164c8: addq.w #$1, $4434ae.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x4434ae_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.add_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        pc = 0x1164ce_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_167_unit::combat_body_block_167;

mod combat_body_block_168_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_168(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001164ce: cmpi.w #$6e, $4434ae.l
        e.step()?;
        let source = 0x6e_u32;
        let destination_address = 0x4434ae_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 001164d6: blt.b $11645c
        e.step()?;
        pc = if r.less()? {
            0x11645c_u32
        } else {
            0x1164d8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_168_unit::combat_body_block_168;

mod combat_body_block_169_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_169(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001164d8: move.w d4, $447192.l
        e.step()?;
        let value = r.data(4, 16)?;
        let destination_address = 0x447192_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001164de: move.w d4, d0
        e.step()?;
        let value = r.data(4, 16)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001164e0: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001164e4: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001164ea: move.w #$6d, $20(a0, d0.l)
        e.step()?;
        let value = 0x6d_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x20_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001164f0: subq.w #$1, $447194.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x447194_u32;
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001164f6: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001164fa: bsr.l $125a3a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116500_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x125a3a_u32)?;
        // 00116500: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00116502: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_169_unit::combat_body_block_169;
mod combat_body_block_16a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_16a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116506: move.w #$3b, -(a7)
        e.step()?;
        let value = 0x3b_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011650a: bsr.l $16474a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116510_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16474a_u32)?;
        // 00116510: move.l $43878c.l, -(a7)
        e.step()?;
        let value_address = 0x43878c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116516: MacToolbox 0xa873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 00116518: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0011651a: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_16a_unit::combat_body_block_16a;

mod combat_body_block_16b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_16b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011651e: move.w $4471ee.l, d0
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116524: asr.w #$4, d0
        e.step()?;
        let count = 0x4_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(0, 16, value);
        // 00116526: lsr.w #$8, d0
        e.step()?;
        let count = 0x8_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(0, 16, value);
        // 00116528: lsr.w #$3, d0
        e.step()?;
        let count = 0x3_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(0, 16, value);
        // 0011652a: add.w $4471ee.l, d0
        e.step()?;
        let source_address = 0x4471ee_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116530: asr.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(0, 16, value);
        // 00116532: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116534: move.w $4471f0.l, d0
        e.step()?;
        let value_address = 0x4471f0_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011653a: asr.w #$4, d0
        e.step()?;
        let count = 0x4_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(0, 16, value);
        // 0011653c: lsr.w #$8, d0
        e.step()?;
        let count = 0x8_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(0, 16, value);
        // 0011653e: lsr.w #$3, d0
        e.step()?;
        let count = 0x3_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, false);
        r.put_data(0, 16, value);
        // 00116540: add.w $4471f0.l, d0
        e.step()?;
        let source_address = 0x4471f0_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116546: asr.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_right(source, count & 63, 16, true);
        r.put_data(0, 16, value);
        // 00116548: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011654a: bsr.l $10e6b6
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116550_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x10e6b6_u32)?;
        // 00116550: move.w #$2791, -(a7)
        e.step()?;
        let value = 0x2791_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116554: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11655a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 0011655a: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0011655c: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_16b_unit::combat_body_block_16b;

mod combat_body_block_16c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_16c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116560: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116566: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 00116568: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0011656e: addi.w #$184, d1
        e.step()?;
        let source = 0x184_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116572: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116574: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011657a: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 0011657c: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116582: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116588: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011658e: addi.w #$1ae, d0
        e.step()?;
        let source = 0x1ae_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116592: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116598: moveq #$23, d0
        e.step()?;
        r.put_data(0, 32, 0x23_u32);
        r.move_flags(0x23_u32, 32);
        // 0011659a: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001165a0: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001165a6: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001165aa: bsr.l $153720
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1165b0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153720_u32)?;
        // 001165b0: move.l $44292c.l, -$18(a6)
        e.step()?;
        let value_address = 0x44292c_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001165b8: move.l $442930.l, -$14(a6)
        e.step()?;
        let value_address = 0x442930_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001165c0: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001165c4: bsr.l $110644
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1165ca_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x110644_u32)?;
        // 001165ca: move.l -$18(a6), $44292c.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
        let value = e.l(value_address)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001165d2: move.l -$14(a6), $442930.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        let value = e.l(value_address)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001165da: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001165de: bsr.l $1537ee
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1165e4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1537ee_u32)?;
        // 001165e4: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001165e6: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_16c_unit::combat_body_block_16c;

mod combat_body_block_16d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_16d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001165ea: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001165f0: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 001165f2: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001165f8: addi.w #$184, d1
        e.step()?;
        let source = 0x184_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 001165fc: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 001165fe: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116604: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 00116606: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011660c: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116612: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116618: addi.w #$1d8, d0
        e.step()?;
        let source = 0x1d8_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011661c: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116622: moveq #$2f, d0
        e.step()?;
        r.put_data(0, 32, 0x2f_u32);
        r.move_flags(0x2f_u32, 32);
        // 00116624: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011662a: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116630: move.l $44292c.l, -$18(a6)
        e.step()?;
        let value_address = 0x44292c_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116638: move.l $442930.l, -$14(a6)
        e.step()?;
        let value_address = 0x442930_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116640: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116644: bsr.l $153720
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11664a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153720_u32)?;
        // 0011664a: move.w #$ffff, -(a7)
        e.step()?;
        let value = 0xffff_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011664e: bsr.l $110644
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116654_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x110644_u32)?;
        // 00116654: move.l -$18(a6), $44292c.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
        let value = e.l(value_address)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011665c: move.l -$14(a6), $442930.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        let value = e.l(value_address)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116664: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116668: bsr.l $1537ee
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11666e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1537ee_u32)?;
        // 0011666e: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00116670: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_16d_unit::combat_body_block_16d;

mod combat_body_block_16e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_16e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116674: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011667a: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0011667c: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00116682: addi.w #$170, d1
        e.step()?;
        let source = 0x170_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116686: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116688: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011668e: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 00116690: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116696: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011669c: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001166a2: addi.w #$1d8, d0
        e.step()?;
        let source = 0x1d8_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001166a6: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001166ac: moveq #$2f, d0
        e.step()?;
        r.put_data(0, 32, 0x2f_u32);
        r.move_flags(0x2f_u32, 32);
        // 001166ae: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001166b4: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001166ba: move.l $44292c.l, -$18(a6)
        e.step()?;
        let value_address = 0x44292c_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001166c2: move.l $442930.l, -$14(a6)
        e.step()?;
        let value_address = 0x442930_u32;
        let value = e.l(value_address)?;
        let destination_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001166ca: move.w #$93, -(a7)
        e.step()?;
        let value = 0x93_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001166ce: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1166d4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 001166d4: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001166d8: bsr.l $153720
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1166de_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153720_u32)?;
        // 001166de: move.w $447194.l, $4435c2.l
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4435c2_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001166e8: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001166ea: bsr.l $110644
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1166f0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x110644_u32)?;
        // 001166f0: move.l -$18(a6), $44292c.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffe8_u32);
        let value = e.l(value_address)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001166f8: move.l -$14(a6), $442930.l
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xffffffec_u32);
        let value = e.l(value_address)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116700: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116704: bsr.l $1537ee
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11670a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1537ee_u32)?;
        // 0011670a: addq.w #$8, a7
        e.step()?;
        let source = 0x8_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0011670c: bra.w $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_16e_unit::combat_body_block_16e;

mod combat_body_block_16f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_16f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116710: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116716: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 00116718: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0011671e: addi.w #$148, d1
        e.step()?;
        let source = 0x148_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116722: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116724: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011672a: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 0011672c: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116732: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116738: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011673e: addi.w #$1d8, d0
        e.step()?;
        let source = 0x1d8_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116742: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116748: moveq #$2f, d0
        e.step()?;
        r.put_data(0, 32, 0x2f_u32);
        r.move_flags(0x2f_u32, 32);
        // 0011674a: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116750: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116756: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011675a: bsr.l $153720
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116760_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153720_u32)?;
        // 00116760: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116766: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011676a: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00116770: clr.w $16(a0, d0.l)
        e.step()?;
        let value = 0;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x16_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116774: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011677a: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011677e: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00116784: clr.w $3c(a0, d0.l)
        e.step()?;
        let value = 0;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x3c_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116788: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011678a: bsr.l $125a3a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116790_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x125a3a_u32)?;
        // 00116790: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00116792: bra.b $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_16f_unit::combat_body_block_16f;

mod combat_body_block_170_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_170(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116794: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011679a: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0011679c: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001167a2: addi.w #$148, d1
        e.step()?;
        let source = 0x148_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 001167a6: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 001167a8: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001167ae: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 001167b0: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001167b6: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001167bc: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001167c2: addi.w #$20c, d0
        e.step()?;
        let source = 0x20c_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001167c6: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001167cc: moveq #$29, d0
        e.step()?;
        r.put_data(0, 32, 0x29_u32);
        r.move_flags(0x29_u32, 32);
        // 001167ce: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001167d4: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001167da: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001167de: bsr.l $153720
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1167e4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x153720_u32)?;
        // 001167e4: move.l $4393f8.l, $439404.l
        e.step()?;
        let value_address = 0x4393f8_u32;
        let value = e.l(value_address)?;
        let destination_address = 0x439404_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001167ee: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001167f0: bra.b $1167fc
        e.step()?;
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_170_unit::combat_body_block_170;

mod combat_body_block_171_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_171(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001167f2: move.l $43942c.l, $439404.l
        e.step()?;
        let value_address = 0x43942c_u32;
        let value = e.l(value_address)?;
        let destination_address = 0x439404_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        pc = 0x1167fc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_171_unit::combat_body_block_171;

mod combat_body_block_172_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_172(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001167fc: tst.b $437a6b.l
        e.step()?;
        let value_address = 0x437a6b_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00116802: beq.w $116b2c
        e.step()?;
        pc = if r.equal()? {
            0x116b2c_u32
        } else {
            0x116806_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_172_unit::combat_body_block_172;

mod combat_body_block_173_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_173(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116806: cmpi.w #$6, $4435d2.l
        e.step()?;
        let source = 0x6_u32;
        let destination_address = 0x4435d2_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 0011680e: bge.w $1168d2
        e.step()?;
        pc = if r.greater_equal()? {
            0x1168d2_u32
        } else {
            0x116812_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_173_unit::combat_body_block_173;

mod combat_body_block_174_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_174(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116812: moveq #$50, d0
        e.step()?;
        r.put_data(0, 32, 0x50_u32);
        r.move_flags(0x50_u32, 32);
        // 00116814: mulu.w $4435d2.l, d0
        e.step()?;
        let source_address = 0x4435d2_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011681a: lea.l $43788a.l, a0
        e.step()?;
        let value = 0x43788a_u32;
        r.put_address(0, value);
        // 00116820: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00116822: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00116824: move.b $437a6b.l, d0
        e.step()?;
        let value_address = 0x437a6b_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0011682a: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011682c: move.w (a0, d0.w * 8), $4435c6.l
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(8));
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4435c6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116834: moveq #$50, d0
        e.step()?;
        r.put_data(0, 32, 0x50_u32);
        r.move_flags(0x50_u32, 32);
        // 00116836: mulu.w $4435d2.l, d0
        e.step()?;
        let source_address = 0x4435d2_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011683c: lea.l $43788a.l, a0
        e.step()?;
        let value = 0x43788a_u32;
        r.put_address(0, value);
        // 00116842: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00116844: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00116846: move.b $437a6b.l, d0
        e.step()?;
        let value_address = 0x437a6b_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0011684c: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011684e: move.w $2(a0, d0.w * 8), $4435c8.l
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(8));
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4435c8_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116856: moveq #$50, d0
        e.step()?;
        r.put_data(0, 32, 0x50_u32);
        r.move_flags(0x50_u32, 32);
        // 00116858: mulu.w $4435d2.l, d0
        e.step()?;
        let source_address = 0x4435d2_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011685e: lea.l $43788a.l, a0
        e.step()?;
        let value = 0x43788a_u32;
        r.put_address(0, value);
        // 00116864: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00116866: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00116868: move.b $437a6b.l, d0
        e.step()?;
        let value_address = 0x437a6b_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0011686e: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116870: move.w $4(a0, d0.w * 8), $4435ca.l
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x4_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(8));
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4435ca_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116878: moveq #$50, d0
        e.step()?;
        r.put_data(0, 32, 0x50_u32);
        r.move_flags(0x50_u32, 32);
        // 0011687a: mulu.w $4435d2.l, d0
        e.step()?;
        let source_address = 0x4435d2_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00116880: lea.l $43788a.l, a0
        e.step()?;
        let value = 0x43788a_u32;
        r.put_address(0, value);
        // 00116886: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 00116888: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 0011688a: move.b $437a6b.l, d0
        e.step()?;
        let value_address = 0x437a6b_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116890: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116892: move.w $6(a0, d0.w * 8), $4435cc.l
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x6_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(8));
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4435cc_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011689a: move.w $4435ca.l, -(a7)
        e.step()?;
        let value_address = 0x4435ca_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001168a0: move.w $4435c8.l, -(a7)
        e.step()?;
        let value_address = 0x4435c8_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001168a6: move.w $4435c6.l, -(a7)
        e.step()?;
        let value_address = 0x4435c6_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001168ac: bsr.l $1314fc
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1168b2_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1314fc_u32)?;
        // 001168b2: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001168b8: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001168bc: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001168c2: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 001168c4: addq.l #$1, $a66(a0)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r.address(0)?.wrapping_add(0xa66_u32);
        let destination = e.l(destination_address)?;
        let value = r.add_flags(destination, source, 32, true);
        r.write_memory(e, destination_address, 32, value)?;
        // 001168c8: move.b #$1, $435662.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x435662_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001168d0: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1168d2_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_174_unit::combat_body_block_174;

mod combat_body_block_175_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_175(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001168d2: subq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 001168d4: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001168d6: move.w $447218.l, d0
        e.step()?;
        let value_address = 0x447218_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001168dc: move.l d0, -(a7)
        e.step()?;
        let value = r.data(0, 32)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001168de: pea.l $100.w
        e.step()?;
        let value = 0x100_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001168e2: MacToolbox 0xa858
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa858_u32 as u16))?;
        // 001168e4: move.l (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = e.l(value_address)?;
        r.offset_address(7, 4)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001168e6: beq.w $116a60
        e.step()?;
        pc = if r.equal()? {
            0x116a60_u32
        } else {
            0x1168ea_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_175_unit::combat_body_block_175;

mod combat_body_block_176_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_176(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001168ea: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001168f0: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001168f4: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001168fa: move.w $58(a0, d0.l), -$24(a6)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x58_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let value = u32::from(e.w(value_address)?);
        let destination_address = r.address(6)?.wrapping_add(0xffffffdc_u32);
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116900: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00116902: move.b $44376c.l, d0
        e.step()?;
        let value_address = 0x44376c_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116908: muls.w $4435cc.l, d0
        e.step()?;
        let source_address = 0x4435cc_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011690e: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116910: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116916_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 00116916: cmp.w -$24(a6), d0
        e.step()?;
        let source_address = r.address(6)?.wrapping_add(0xffffffdc_u32);
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 0011691a: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0011691c: ble.b $11692c
        e.step()?;
        pc = if r.less_equal()? {
            0x11692c_u32
        } else {
            0x11691e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_176_unit::combat_body_block_176;

mod combat_body_block_177_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_177(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011691e: move.l $439470.l, $439404.l
        e.step()?;
        let value_address = 0x439470_u32;
        let value = e.l(value_address)?;
        let destination_address = 0x439404_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116928: bra.w $116b2c
        e.step()?;
        pc = 0x116b2c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_177_unit::combat_body_block_177;

mod combat_body_block_178_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_178(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011692c: tst.w $438776.l
        e.step()?;
        let value_address = 0x438776_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00116932: beq.b $11694c
        e.step()?;
        pc = if r.equal()? {
            0x11694c_u32
        } else {
            0x116934_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_178_unit::combat_body_block_178;

mod combat_body_block_179_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_179(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116934: tst.b $443763.l
        e.step()?;
        let value_address = 0x443763_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0011693a: bne.b $11694c
        e.step()?;
        pc = if r.not_equal()? {
            0x11694c_u32
        } else {
            0x11693c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_179_unit::combat_body_block_179;
mod combat_body_block_17a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_17a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011693c: move.w #$8f, -(a7)
        e.step()?;
        let value = 0x8f_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116940: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116946_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00116946: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00116948: bra.w $116b2c
        e.step()?;
        pc = 0x116b2c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_17a_unit::combat_body_block_17a;

mod combat_body_block_17b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_17b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011694c: tst.w $438776.l
        e.step()?;
        let value_address = 0x438776_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00116952: bne.b $11696c
        e.step()?;
        pc = if r.not_equal()? {
            0x11696c_u32
        } else {
            0x116954_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_17b_unit::combat_body_block_17b;

mod combat_body_block_17c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_17c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116954: tst.b $443764.l
        e.step()?;
        let value_address = 0x443764_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0011695a: bne.b $11696c
        e.step()?;
        pc = if r.not_equal()? {
            0x11696c_u32
        } else {
            0x11695c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_17c_unit::combat_body_block_17c;

mod combat_body_block_17d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_17d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011695c: move.w #$8f, -(a7)
        e.step()?;
        let value = 0x8f_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116960: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116966_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00116966: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00116968: bra.w $116b2c
        e.step()?;
        pc = 0x116b2c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_17d_unit::combat_body_block_17d;

mod combat_body_block_17e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_17e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011696c: tst.w $4435cc.l
        e.step()?;
        let value_address = 0x4435cc_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00116972: bne.b $116984
        e.step()?;
        pc = if r.not_equal()? {
            0x116984_u32
        } else {
            0x116974_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_17e_unit::combat_body_block_17e;

mod combat_body_block_17f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_17f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116974: move.w #$8f, -(a7)
        e.step()?;
        let value = 0x8f_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116978: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11697e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 0011697e: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00116980: bra.w $116b2c
        e.step()?;
        pc = 0x116b2c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_17f_unit::combat_body_block_17f;

mod combat_body_block_180_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_180(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116984: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00116986: move.b $44376c.l, d0
        e.step()?;
        let value_address = 0x44376c_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0011698c: muls.w $4435cc.l, d0
        e.step()?;
        let source_address = 0x4435cc_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00116992: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116994: jsr $400038.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11699a_u32)?;
        imported(e, r, NativeCall::Library(0x400038_u32))?;
        // 0011699a: move.w $4435d2.l, d1
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 001169a0: mulu.w #$c42, d1
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(1, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(1, 32, value);
        r.move_flags(value, 32);
        // 001169a4: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 001169aa: lea.l $58(a0, d1.l), a0
        e.step()?;
        let value = r
            .address(0)?
            .wrapping_add(0x58_u32)
            .wrapping_add((r.data(1, 32)?).wrapping_mul(1));
        r.put_address(0, value);
        // 001169ae: sub.w d0, (a0)
        e.step()?;
        let source = r.data(0, 16)?;
        let destination_address = r.address(0)?.wrapping_add(0x0_u32);
        let destination = u32::from(e.w(destination_address)?);
        let value = r.subtract_flags(destination, source, 16, true);
        r.write_memory(e, destination_address, 16, value)?;
        // 001169b0: lea.l $442580.l, a0
        e.step()?;
        let value = 0x442580_u32;
        r.put_address(0, value);
        // 001169b6: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001169bc: move.b $4435c9.l, (a0, d0.w * 2)
        e.step()?;
        let value_address = 0x4435c9_u32;
        let value = u32::from(e.b(value_address)?);
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001169c4: lea.l $442580.l, a0
        e.step()?;
        let value = 0x442580_u32;
        r.put_address(0, value);
        // 001169ca: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001169d0: move.b $4435cb.l, $1(a0, d0.w * 2)
        e.step()?;
        let value_address = 0x4435cb_u32;
        let value = u32::from(e.b(value_address)?);
        let destination_address = r
            .address(0)?
            .wrapping_add(0x1_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001169d8: lea.l $437a7a.l, a0
        e.step()?;
        let value = 0x437a7a_u32;
        r.put_address(0, value);
        // 001169de: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001169e4: move.w $4435cc.l, (a0, d0.w * 2)
        e.step()?;
        let value_address = 0x4435cc_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001169ec: cmpi.b #$7, $443761.l
        e.step()?;
        let source = 0x7_u32;
        let destination_address = 0x443761_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 001169f4: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001169f6: ble.b $116a0e
        e.step()?;
        pc = if r.less_equal()? {
            0x116a0e_u32
        } else {
            0x1169f8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_180_unit::combat_body_block_180;

mod combat_body_block_181_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_181(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001169f8: move.w $453688.l, $4435cc.l
        e.step()?;
        let value_address = 0x453688_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4435cc_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116a02: move.w $453688.l, $4435be.l
        e.step()?;
        let value_address = 0x453688_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4435be_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116a0c: bra.b $116a36
        e.step()?;
        pc = 0x116a36_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_181_unit::combat_body_block_181;

mod combat_body_block_182_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_182(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116a0e: cmpi.b #$1, $44374f.l
        e.step()?;
        let source = 0x1_u32;
        let destination_address = 0x44374f_u32;
        let destination = u32::from(e.b(destination_address)?);
        r.subtract_flags(destination, source, 8, false);
        // 00116a16: bge.b $116a2e
        e.step()?;
        pc = if r.greater_equal()? {
            0x116a2e_u32
        } else {
            0x116a18_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_182_unit::combat_body_block_182;

mod combat_body_block_183_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_183(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116a18: move.w $453688.l, $4435cc.l
        e.step()?;
        let value_address = 0x453688_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4435cc_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116a22: move.w $453688.l, $4435be.l
        e.step()?;
        let value_address = 0x453688_u32;
        let value = u32::from(e.w(value_address)?);
        let destination_address = 0x4435be_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116a2c: bra.b $116a36
        e.step()?;
        pc = 0x116a36_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_183_unit::combat_body_block_183;

mod combat_body_block_184_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_184(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116a2e: move.w #$1, $4435be.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x4435be_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x116a36_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_184_unit::combat_body_block_184;

mod combat_body_block_185_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_185(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116a36: move.b #$1, $437a6a.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x437a6a_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00116a3e: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00116a40: moveq #$1, d0
        e.step()?;
        r.put_data(0, 32, 0x1_u32);
        r.move_flags(0x1_u32, 32);
        // 00116a42: lea.l $442798.l, a0
        e.step()?;
        let value = 0x442798_u32;
        r.put_address(0, value);
        // 00116a48: move.w $4435d2.l, d1
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00116a4e: move.w d0, (a0, d1.w * 2)
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(1, 16)? as u16 as i16) as u32).wrapping_mul(2));
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116a52: move.l $439470.l, $439404.l
        e.step()?;
        let value_address = 0x439470_u32;
        let value = e.l(value_address)?;
        let destination_address = 0x439404_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116a5c: bra.w $116b2c
        e.step()?;
        pc = 0x116b2c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_185_unit::combat_body_block_185;

mod combat_body_block_186_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_186(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116a60: move.l $43878c.l, -(a7)
        e.step()?;
        let value_address = 0x43878c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116a66: MacToolbox 0xa873
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa873_u32 as u16))?;
        // 00116a68: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116a6e: addi.w #$1a9, d0
        e.step()?;
        let source = 0x1a9_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116a72: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116a74: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116a7a: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 00116a7c: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00116a82: addi.w #$1b8, d1
        e.step()?;
        let source = 0x1b8_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116a86: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116a88: move.w d1, -(a7)
        e.step()?;
        let value = r.data(1, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116a8a: MacToolbox 0xa893
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa893_u32 as u16))?;
        // 00116a8c: move.w #$1, -(a7)
        e.step()?;
        let value = 0x1_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116a90: MacToolbox 0xa889
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa889_u32 as u16))?;
        // 00116a92: move.w #$e, -(a7)
        e.step()?;
        let value = 0xe_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116a96: MacToolbox 0xa88a
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa88a_u32 as u16))?;
        // 00116a98: move.l $44370e.l, -(a7)
        e.step()?;
        let value_address = 0x44370e_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116a9e: move.l $44370a.l, -(a7)
        e.step()?;
        let value_address = 0x44370a_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116aa4: move.w #$9c, -(a7)
        e.step()?;
        let value = 0x9c_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116aa8: bsr.l $1382a8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116aae_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1382a8_u32)?;
        // 00116aae: pea.l $45.w
        e.step()?;
        let value = 0x45_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116ab2: MacToolbox 0xa862
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa862_u32 as u16))?;
        // 00116ab4: pea.l $479ed8.l
        e.step()?;
        let value = 0x479ed8_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116aba: move.w $4435c6.l, d0
        e.step()?;
        let value_address = 0x4435c6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116ac0: addq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116ac2: muls.w #$3e8, d0
        e.step()?;
        let source = 0x3e8_u32;
        let destination = r.data(0, 16)?;
        let value = i32::from(source as u16 as i16)
            .wrapping_mul(i32::from(destination as u16 as i16)) as u32;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00116ac6: add.w $4435c8.l, d0
        e.step()?;
        let source_address = 0x4435c8_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116acc: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116ace: move.w $4435ca.l, d0
        e.step()?;
        let value_address = 0x4435ca_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116ad4: addq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116ad6: move.w d0, -(a7)
        e.step()?;
        let value = r.data(0, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116ad8: jsr $400178.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116ade_u32)?;
        imported(e, r, NativeCall::Library(0x400178_u32))?;
        // 00116ade: tst.w $4435cc.l
        e.step()?;
        let value_address = 0x4435cc_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00116ae4: lea.l $a(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xa_u32);
        r.put_address(7, value);
        // 00116ae8: beq.b $116b18
        e.step()?;
        pc = if r.equal()? {
            0x116b18_u32
        } else {
            0x116aea_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_186_unit::combat_body_block_186;

mod combat_body_block_187_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_187(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116aea: pea.l $479ed8.l
        e.step()?;
        let value = 0x479ed8_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116af0: MacToolbox 0xa884
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa884_u32 as u16))?;
        // 00116af2: pea.l $403dd0.l
        e.step()?;
        let value = 0x403dd0_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116af8: MacToolbox 0xa884
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa884_u32 as u16))?;
        // 00116afa: movea.w $4435cc.l, a0
        e.step()?;
        let value_address = 0x4435cc_u32;
        let value = u32::from(e.w(value_address)?);
        let value = i32::from(value as u16 as i16) as u32;
        r.put_address(0, value);
        // 00116b00: move.l a0, -(a7)
        e.step()?;
        let value = r.address(0)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116b02: pea.l $479ed8.l
        e.step()?;
        let value = 0x479ed8_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116b08: jsr $400168.l
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116b0e_u32)?;
        imported(e, r, NativeCall::Library(0x400168_u32))?;
        // 00116b0e: pea.l $479ed8.l
        e.step()?;
        let value = 0x479ed8_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116b14: MacToolbox 0xa884
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa884_u32 as u16))?;
        // 00116b16: bra.b $116b2c
        e.step()?;
        pc = 0x116b2c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_187_unit::combat_body_block_187;

mod combat_body_block_188_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_188(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116b18: pea.l $403dc0.l
        e.step()?;
        let value = 0x403dc0_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116b1e: MacToolbox 0xa884
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa884_u32 as u16))?;
        // 00116b20: move.w #$ff71, -(a7)
        e.step()?;
        let value = 0xff71_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116b24: bsr.l $133a18
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116b2a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x133a18_u32)?;
        // 00116b2a: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x116b2c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_188_unit::combat_body_block_188;

mod combat_body_block_189_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_189(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116b2c: tst.l $439404.l
        e.step()?;
        let value_address = 0x439404_u32;
        let value = e.l(value_address)?;
        r.move_flags(value, 32);
        // 00116b32: beq.w $117338
        e.step()?;
        pc = if r.equal()? {
            0x117338_u32
        } else {
            0x116b36_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_189_unit::combat_body_block_189;
mod combat_body_block_18a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_18a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116b36: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 00116b3c: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116b42: cmpi.w #$9, (a0, d0.w * 2)
        e.step()?;
        let source = 0x9_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00116b48: bge.w $117338
        e.step()?;
        pc = if r.greater_equal()? {
            0x117338_u32
        } else {
            0x116b4c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_18a_unit::combat_body_block_18a;

mod combat_body_block_18b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_18b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116b4c: move.b $442739.l, d0
        e.step()?;
        let value_address = 0x442739_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116b52: cmp.b $44273a.l, d0
        e.step()?;
        let source_address = 0x44273a_u32;
        let source = u32::from(e.b(source_address)?);
        let destination = r.data(0, 8)?;
        r.subtract_flags(destination, source, 8, false);
        // 00116b58: ble.b $116b64
        e.step()?;
        pc = if r.less_equal()? {
            0x116b64_u32
        } else {
            0x116b5a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_18b_unit::combat_body_block_18b;

mod combat_body_block_18c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_18c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116b5a: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116b5c: bsr.l $14913e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116b62_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x14913e_u32)?;
        // 00116b62: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x116b64_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_18c_unit::combat_body_block_18c;

mod combat_body_block_18d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_18d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116b64: bsr.l $117be2
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116b6a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x117be2_u32)?;
        // 00116b6a: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116b70: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00116b74: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00116b7a: cmpi.w #$2, $40(a0, d0.l)
        e.step()?;
        let source = 0x2_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x40_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00116b80: bge.w $117338
        e.step()?;
        pc = if r.greater_equal()? {
            0x117338_u32
        } else {
            0x116b84_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_18d_unit::combat_body_block_18d;

mod combat_body_block_18e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_18e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116b84: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116b86: bsr.l $125a3a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116b8c_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x125a3a_u32)?;
        // 00116b8c: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00116b8e: bra.w $117338
        e.step()?;
        pc = 0x117338_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_18e_unit::combat_body_block_18e;

mod combat_body_block_18f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_18f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116b92: cmpi.w #$100, $447218.l
        e.step()?;
        let source = 0x100_u32;
        let destination_address = 0x447218_u32;
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00116b9a: bne.b $116bc2
        e.step()?;
        pc = if r.not_equal()? {
            0x116bc2_u32
        } else {
            0x116b9c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_18f_unit::combat_body_block_18f;

mod combat_body_block_190_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_190(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116b9c: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00116b9e: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116ba4: pea.l $4471b4.l
        e.step()?;
        let value = 0x4471b4_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116baa: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 00116bac: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116bae: beq.b $116bc2
        e.step()?;
        pc = if r.equal()? {
            0x116bc2_u32
        } else {
            0x116bb0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_190_unit::combat_body_block_190;

mod combat_body_block_191_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_191(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116bb0: move.w $4435d2.l, -(a7)
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116bb6: bsr.l $117a54
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116bbc_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x117a54_u32)?;
        // 00116bbc: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00116bbe: bra.w $11597c
        e.step()?;
        pc = 0x11597c_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_191_unit::combat_body_block_191;

mod combat_body_block_192_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_192(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116bc2: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00116bc4: move.l $447214.l, -(a7)
        e.step()?;
        let value_address = 0x447214_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116bca: pea.l -$1c(a6)
        e.step()?;
        let value = r.address(6)?.wrapping_add(0xffffffe4_u32);
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116bce: MacToolbox 0xa92c
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa92c_u32 as u16))?;
        // 00116bd0: move.w (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116bd2: move.w d0, $4471a6.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4471a6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116bd8: move.w $4471a6.l, d0
        e.step()?;
        let value_address = 0x4471a6_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116bde: subq.w #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116be0: beq.b $116bea
        e.step()?;
        pc = if r.equal()? {
            0x116bea_u32
        } else {
            0x116be2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_192_unit::combat_body_block_192;

mod combat_body_block_193_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_193(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116be2: subq.w #$2, d0
        e.step()?;
        let source = 0x2_u32;
        let destination = r.data(0, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116be4: beq.b $116c18
        e.step()?;
        pc = if r.equal()? {
            0x116c18_u32
        } else {
            0x116be6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_193_unit::combat_body_block_193;

mod combat_body_block_194_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_194(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116be6: bra.w $117338
        e.step()?;
        pc = 0x117338_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_194_unit::combat_body_block_194;

mod combat_body_block_195_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_195(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116bea: bsr.l $13e13c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116bf0_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13e13c_u32)?;
        // 00116bf0: subq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00116bf2: move.l $447214.l, -(a7)
        e.step()?;
        let value_address = 0x447214_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116bf8: MacToolbox 0xa93d
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa93d_u32 as u16))?;
        // 00116bfa: move.l (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = e.l(value_address)?;
        r.offset_address(7, 4)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00116bfc: move.l d0, $438742.l
        e.step()?;
        let value = r.data(0, 32)?;
        let destination_address = 0x438742_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116c02: bsr.l $1279bc
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116c08_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1279bc_u32)?;
        // 00116c08: tst.b $4386b2.l
        e.step()?;
        let value_address = 0x4386b2_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00116c0e: beq.w $117338
        e.step()?;
        pc = if r.equal()? {
            0x117338_u32
        } else {
            0x116c12_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_195_unit::combat_body_block_195;

mod combat_body_block_196_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_196(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116c12: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 00116c14: bra.w $117344
        e.step()?;
        pc = 0x117344_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_196_unit::combat_body_block_196;

mod combat_body_block_197_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_197(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116c18: lea.l $4470b8.l, a0
        e.step()?;
        let value = 0x4470b8_u32;
        r.put_address(0, value);
        // 00116c1e: move.w $447194.l, d0
        e.step()?;
        let value_address = 0x447194_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116c24: cmpi.w #$9, (a0, d0.w * 2)
        e.step()?;
        let source = 0x9_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(2));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00116c2a: bge.w $117338
        e.step()?;
        pc = if r.greater_equal()? {
            0x117338_u32
        } else {
            0x116c2e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_197_unit::combat_body_block_197;

mod combat_body_block_198_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_198(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116c2e: bra.w $116d58
        e.step()?;
        pc = 0x116d58_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_198_unit::combat_body_block_198;

mod combat_body_block_199_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_199(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116c32: pea.l $4471ee.l
        e.step()?;
        let value = 0x4471ee_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116c38: MacToolbox 0xa972
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa972_u32 as u16))?;
        // 00116c3a: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00116c3c: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116c42: pea.l $4471b4.l
        e.step()?;
        let value = 0x4471b4_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116c48: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 00116c4a: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116c4c: bne.b $116c56
        e.step()?;
        pc = if r.not_equal()? {
            0x116c56_u32
        } else {
            0x116c4e_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_199_unit::combat_body_block_199;
mod combat_body_block_19a_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_19a(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116c4e: tst.b -$b(a6)
        e.step()?;
        let value_address = r.address(6)?.wrapping_add(0xfffffff5_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00116c52: beq.w $116d84
        e.step()?;
        pc = if r.equal()? {
            0x116d84_u32
        } else {
            0x116c56_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_19a_unit::combat_body_block_19a;

mod combat_body_block_19b_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_19b(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116c56: tst.b $43565c.l
        e.step()?;
        let value_address = 0x43565c_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00116c5c: bne.w $116d84
        e.step()?;
        pc = if r.not_equal()? {
            0x116d84_u32
        } else {
            0x116c60_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_19b_unit::combat_body_block_19b;

mod combat_body_block_19c_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_19c(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116c60: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116c62: bsr.l $15f210
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116c68_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x15f210_u32)?;
        // 00116c68: move.w $4471a2.l, d0
        e.step()?;
        let value_address = 0x4471a2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116c6e: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00116c70: cmpi.w #$8, d0
        e.step()?;
        let source = 0x8_u32;
        let destination = r.data(0, 16)?;
        r.subtract_flags(destination, source, 16, false);
        // 00116c74: bhi.b $116cde
        e.step()?;
        pc = if r.higher()? {
            0x116cde_u32
        } else {
            0x116c76_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_19c_unit::combat_body_block_19c;

mod combat_body_block_19d_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_19d(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116c76: move.w $116c7e(pc, d0.w), d0
        e.step()?;
        let selector = r.data(0, 16)? as usize;
        let offsets: [u16; 9] = [
            0x0012, 0x001c, 0x0026, 0x0030, 0x0060, 0x003a, 0x0044, 0x004e, 0x0058,
        ];
        let value = u32::from(*offsets.get(selector).ok_or_else(|| {
            RuleError::Invalid("source event selector outside recovered table".into())
        })?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116c7a: jmp $116c7e(pc, d0.w)
        e.step()?;
        pc = 0x116c7e_u32.wrapping_add(i32::from(r.data(0, 16)? as u16 as i16) as u32);

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_19d_unit::combat_body_block_19d;

mod combat_body_block_19e_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_19e(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116c90: move.w #$5937, $443316.l
        e.step()?;
        let value = 0x5937_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116c98: bra.b $116cde
        e.step()?;
        pc = 0x116cde_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_19e_unit::combat_body_block_19e;

mod combat_body_block_19f_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_19f(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116c9a: move.w #$5b38, $443316.l
        e.step()?;
        let value = 0x5b38_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116ca2: bra.b $116cde
        e.step()?;
        pc = 0x116cde_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_19f_unit::combat_body_block_19f;

mod combat_body_block_1a0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1a0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116ca4: move.w #$5c39, $443316.l
        e.step()?;
        let value = 0x5c39_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116cac: bra.b $116cde
        e.step()?;
        pc = 0x116cde_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1a0_unit::combat_body_block_1a0;

mod combat_body_block_1a1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1a1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116cae: move.w #$7b1c, $443316.l
        e.step()?;
        let value = 0x7b1c_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116cb6: bra.b $116cde
        e.step()?;
        pc = 0x116cde_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1a1_unit::combat_body_block_1a1;

mod combat_body_block_1a2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1a2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116cb8: move.w #$7c1d, $443316.l
        e.step()?;
        let value = 0x7c1d_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116cc0: bra.b $116cde
        e.step()?;
        pc = 0x116cde_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1a2_unit::combat_body_block_1a2;

mod combat_body_block_1a3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1a3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116cc2: move.w #$5331, $443316.l
        e.step()?;
        let value = 0x5331_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116cca: bra.b $116cde
        e.step()?;
        pc = 0x116cde_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1a3_unit::combat_body_block_1a3;

mod combat_body_block_1a4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1a4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116ccc: move.w #$5432, $443316.l
        e.step()?;
        let value = 0x5432_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116cd4: bra.b $116cde
        e.step()?;
        pc = 0x116cde_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1a4_unit::combat_body_block_1a4;

mod combat_body_block_1a5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1a5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116cd6: move.w #$5533, $443316.l
        e.step()?;
        let value = 0x5533_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x116cde_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1a5_unit::combat_body_block_1a5;

mod combat_body_block_1a6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1a6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116cde: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116ce0: bsr.l $111984
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116ce6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x111984_u32)?;
        // 00116ce6: tst.b $442736.l
        e.step()?;
        let value_address = 0x442736_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00116cec: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00116cee: bne.b $116cf8
        e.step()?;
        pc = if r.not_equal()? {
            0x116cf8_u32
        } else {
            0x116cf0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1a6_unit::combat_body_block_1a6;

mod combat_body_block_1a7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1a7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116cf0: tst.b $442737.l
        e.step()?;
        let value_address = 0x442737_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00116cf6: beq.b $116d58
        e.step()?;
        pc = if r.equal()? {
            0x116d58_u32
        } else {
            0x116cf8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1a7_unit::combat_body_block_1a7;

mod combat_body_block_1a8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1a8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116cf8: tst.b $43565f.l
        e.step()?;
        let value_address = 0x43565f_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00116cfe: bne.b $116d58
        e.step()?;
        pc = if r.not_equal()? {
            0x116d58_u32
        } else {
            0x116d00_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1a8_unit::combat_body_block_1a8;

mod combat_body_block_1a9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1a9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116d00: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116d06: add.w d0, d0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116d08: move.b $442737.l, d1
        e.step()?;
        let value_address = 0x442737_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(1, 8, value);
        r.move_flags(value, 8);
        // 00116d0e: ext.w d1
        e.step()?;
        let source = r.data(1, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00116d10: addq.w #$5, d1
        e.step()?;
        let source = 0x5_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116d12: add.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116d14: move.w d1, -(a7)
        e.step()?;
        let value = r.data(1, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116d16: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116d1c: add.w d0, d0
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116d1e: move.b $442736.l, d1
        e.step()?;
        let value_address = 0x442736_u32;
        let value = u32::from(e.b(value_address)?);
        r.put_data(1, 8, value);
        r.move_flags(value, 8);
        // 00116d24: ext.w d1
        e.step()?;
        let source = r.data(1, 8)?;
        let value = i32::from(source as u8 as i8) as u32;
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00116d26: addq.w #$5, d1
        e.step()?;
        let source = 0x5_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116d28: add.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116d2a: move.w d1, -(a7)
        e.step()?;
        let value = r.data(1, 16)?;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116d2c: bsr.l $10e6b6
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116d32_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x10e6b6_u32)?;
        // 00116d32: move.b #$1, -$b(a6)
        e.step()?;
        let value = 0x1_u32;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff5_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00116d38: move.b #$1, $43565b.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x43565b_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00116d40: clr.b $442737.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442737_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00116d46: clr.b $442736.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442736_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00116d4c: move.w #$4, -(a7)
        e.step()?;
        let value = 0x4_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116d50: bsr.l $1205f8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x116d56_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x1205f8_u32)?;
        // 00116d56: addq.w #$6, a7
        e.step()?;
        let source = 0x6_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x116d58_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1a9_unit::combat_body_block_1a9;

mod combat_body_block_1aa_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1aa(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116d58: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00116d5a: MacToolbox 0xa973
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa973_u32 as u16))?;
        // 00116d5c: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116d5e: bne.w $116c32
        e.step()?;
        pc = if r.not_equal()? {
            0x116c32_u32
        } else {
            0x116d62_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1aa_unit::combat_body_block_1aa;

mod combat_body_block_1ab_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1ab(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116d62: tst.b $43565b.l
        e.step()?;
        let value_address = 0x43565b_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 00116d68: beq.b $116d84
        e.step()?;
        pc = if r.equal()? {
            0x116d84_u32
        } else {
            0x116d6a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1ab_unit::combat_body_block_1ab;

mod combat_body_block_1ac_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1ac(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116d6a: clr.b $442737.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442737_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00116d70: clr.b $442736.l
        e.step()?;
        let value = 0;
        let destination_address = 0x442736_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00116d76: clr.b $43565b.l
        e.step()?;
        let value = 0;
        let destination_address = 0x43565b_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00116d7c: clr.b -$b(a6)
        e.step()?;
        let value = 0;
        let destination_address = r.address(6)?.wrapping_add(0xfffffff5_u32);
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 00116d80: bra.w $117338
        e.step()?;
        pc = 0x117338_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1ac_unit::combat_body_block_1ac;

mod combat_body_block_1ad_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1ad(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116d84: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00116d86: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116d8c: move.l $43878c.l, -(a7)
        e.step()?;
        let value_address = 0x43878c_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116d92: pea.l $439404.l
        e.step()?;
        let value = 0x439404_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116d98: MacToolbox 0xa96c
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa96c_u32 as u16))?;
        // 00116d9a: move.w (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116d9c: move.w d0, $4471a6.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x4471a6_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116da2: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00116da4: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116daa: pea.l $43872e.l
        e.step()?;
        let value = 0x43872e_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116db0: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 00116db2: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116db4: beq.w $1171d4
        e.step()?;
        pc = if r.equal()? {
            0x1171d4_u32
        } else {
            0x116db8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1ad_unit::combat_body_block_1ad;

mod combat_body_block_1ae_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1ae(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116db8: clr.w $443316.l
        e.step()?;
        let value = 0;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116dbe: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116dc4: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 00116dc6: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00116dcc: addi.w #$16f, d1
        e.step()?;
        let source = 0x16f_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116dd0: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116dd2: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116dd8: moveq #$22, d0
        e.step()?;
        r.put_data(0, 32, 0x22_u32);
        r.move_flags(0x22_u32, 32);
        // 00116dda: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116de0: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116de6: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116dec: addi.w #$20a, d0
        e.step()?;
        let source = 0x20a_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116df0: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116df6: moveq #$22, d0
        e.step()?;
        r.put_data(0, 32, 0x22_u32);
        r.move_flags(0x22_u32, 32);
        // 00116df8: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116dfe: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116e04: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00116e06: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116e0c: pea.l $44292c.l
        e.step()?;
        let value = 0x44292c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116e12: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 00116e14: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116e16: beq.b $116e22
        e.step()?;
        pc = if r.equal()? {
            0x116e22_u32
        } else {
            0x116e18_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1ae_unit::combat_body_block_1ae;

mod combat_body_block_1af_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1af(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116e18: move.l $43943c.l, $439404.l
        e.step()?;
        let value_address = 0x43943c_u32;
        let value = e.l(value_address)?;
        let destination_address = 0x439404_u32;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        pc = 0x116e22_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1af_unit::combat_body_block_1af;

mod combat_body_block_1b0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1b0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116e22: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116e28: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 00116e2a: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00116e30: addi.w #$148, d1
        e.step()?;
        let source = 0x148_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116e34: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116e36: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116e3c: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 00116e3e: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116e44: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116e4a: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116e50: addi.w #$16e, d0
        e.step()?;
        let source = 0x16e_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116e54: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116e5a: moveq #$2e, d0
        e.step()?;
        r.put_data(0, 32, 0x2e_u32);
        r.move_flags(0x2e_u32, 32);
        // 00116e5c: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116e62: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116e68: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00116e6a: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116e70: pea.l $44292c.l
        e.step()?;
        let value = 0x44292c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116e76: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 00116e78: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116e7a: beq.b $116e84
        e.step()?;
        pc = if r.equal()? {
            0x116e84_u32
        } else {
            0x116e7c_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1b0_unit::combat_body_block_1b0;

mod combat_body_block_1b1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1b1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116e7c: move.w #$567, $443316.l
        e.step()?;
        let value = 0x567_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x116e84_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1b1_unit::combat_body_block_1b1;

mod combat_body_block_1b2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1b2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116e84: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116e8a: addi.w #$1d8, d0
        e.step()?;
        let source = 0x1d8_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116e8e: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116e94: moveq #$2f, d0
        e.step()?;
        r.put_data(0, 32, 0x2f_u32);
        r.move_flags(0x2f_u32, 32);
        // 00116e96: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116e9c: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116ea2: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00116ea4: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116eaa: pea.l $44292c.l
        e.step()?;
        let value = 0x44292c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116eb0: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 00116eb2: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116eb4: beq.b $116ebe
        e.step()?;
        pc = if r.equal()? {
            0x116ebe_u32
        } else {
            0x116eb6_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1b2_unit::combat_body_block_1b2;

mod combat_body_block_1b3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1b3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116eb6: move.w #$366, $443316.l
        e.step()?;
        let value = 0x366_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x116ebe_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1b3_unit::combat_body_block_1b3;

mod combat_body_block_1b4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1b4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116ebe: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116ec4: addi.w #$1a3, d0
        e.step()?;
        let source = 0x1a3_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116ec8: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116ece: moveq #$2e, d0
        e.step()?;
        r.put_data(0, 32, 0x2e_u32);
        r.move_flags(0x2e_u32, 32);
        // 00116ed0: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116ed6: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116edc: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00116ede: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116ee4: pea.l $44292c.l
        e.step()?;
        let value = 0x44292c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116eea: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 00116eec: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116eee: beq.b $116ef8
        e.step()?;
        pc = if r.equal()? {
            0x116ef8_u32
        } else {
            0x116ef0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1b4_unit::combat_body_block_1b4;

mod combat_body_block_1b5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1b5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116ef0: move.w #$264, $443316.l
        e.step()?;
        let value = 0x264_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x116ef8_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1b5_unit::combat_body_block_1b5;

mod combat_body_block_1b6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1b6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116ef8: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116efe: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 00116f00: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00116f06: addi.w #$15c, d1
        e.step()?;
        let source = 0x15c_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116f0a: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116f0c: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116f12: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 00116f14: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116f1a: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116f20: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116f26: addi.w #$16e, d0
        e.step()?;
        let source = 0x16e_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116f2a: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116f30: moveq #$63, d0
        e.step()?;
        r.put_data(0, 32, 0x63_u32);
        r.move_flags(0x63_u32, 32);
        // 00116f32: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116f38: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116f3e: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00116f40: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116f46: pea.l $44292c.l
        e.step()?;
        let value = 0x44292c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116f4c: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 00116f4e: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116f50: beq.b $116f5a
        e.step()?;
        pc = if r.equal()? {
            0x116f5a_u32
        } else {
            0x116f52_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1b6_unit::combat_body_block_1b6;

mod combat_body_block_1b7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1b7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116f52: move.w #$f72, $443316.l
        e.step()?;
        let value = 0xf72_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x116f5a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1b7_unit::combat_body_block_1b7;

mod combat_body_block_1b8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1b8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116f5a: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116f60: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 00116f62: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00116f68: addi.w #$184, d1
        e.step()?;
        let source = 0x184_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116f6c: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00116f6e: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116f74: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 00116f76: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116f7c: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116f82: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116f88: addi.w #$1ae, d0
        e.step()?;
        let source = 0x1ae_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116f8c: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116f92: moveq #$23, d0
        e.step()?;
        r.put_data(0, 32, 0x23_u32);
        r.move_flags(0x23_u32, 32);
        // 00116f94: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116f9a: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116fa0: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00116fa2: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116fa8: pea.l $44292c.l
        e.step()?;
        let value = 0x44292c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116fae: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 00116fb0: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116fb2: beq.b $116fbc
        e.step()?;
        pc = if r.equal()? {
            0x116fbc_u32
        } else {
            0x116fb4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1b8_unit::combat_body_block_1b8;

mod combat_body_block_1b9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1b9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116fb4: move.w #$2d6e, $443316.l
        e.step()?;
        let value = 0x2d6e_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x116fbc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1b9_unit::combat_body_block_1b9;

mod combat_body_block_1ba_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1ba(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116fbc: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116fc2: addi.w #$16e, d0
        e.step()?;
        let source = 0x16e_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116fc6: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116fcc: moveq #$39, d0
        e.step()?;
        r.put_data(0, 32, 0x39_u32);
        r.move_flags(0x39_u32, 32);
        // 00116fce: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00116fd4: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00116fda: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00116fdc: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00116fe2: pea.l $44292c.l
        e.step()?;
        let value = 0x44292c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00116fe8: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 00116fea: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00116fec: beq.b $116ff6
        e.step()?;
        pc = if r.equal()? {
            0x116ff6_u32
        } else {
            0x116fee_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1ba_unit::combat_body_block_1ba;

mod combat_body_block_1bb_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1bb(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116fee: move.w #$b62, $443316.l
        e.step()?;
        let value = 0xb62_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x116ff6_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1bb_unit::combat_body_block_1bb;

mod combat_body_block_1bc_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1bc(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00116ff6: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00116ffc: addi.w #$1d6, d0
        e.step()?;
        let source = 0x1d6_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00117000: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00117006: moveq #$32, d0
        e.step()?;
        r.put_data(0, 32, 0x32_u32);
        r.move_flags(0x32_u32, 32);
        // 00117008: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011700e: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00117014: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00117016: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011701c: pea.l $44292c.l
        e.step()?;
        let value = 0x44292c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00117022: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 00117024: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00117026: beq.b $117030
        e.step()?;
        pc = if r.equal()? {
            0x117030_u32
        } else {
            0x117028_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1bc_unit::combat_body_block_1bc;

mod combat_body_block_1bd_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1bd(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00117028: move.w #$2370, $443316.l
        e.step()?;
        let value = 0x2370_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x117030_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1bd_unit::combat_body_block_1bd;

mod combat_body_block_1be_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1be(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00117030: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00117036: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 00117038: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 0011703e: addi.w #$170, d1
        e.step()?;
        let source = 0x170_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00117042: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00117044: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011704a: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 0011704c: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00117052: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00117058: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011705e: addi.w #$1d8, d0
        e.step()?;
        let source = 0x1d8_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00117062: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00117068: moveq #$2f, d0
        e.step()?;
        r.put_data(0, 32, 0x2f_u32);
        r.move_flags(0x2f_u32, 32);
        // 0011706a: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00117070: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00117076: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 00117078: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 0011707e: pea.l $44292c.l
        e.step()?;
        let value = 0x44292c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 00117084: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 00117086: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 00117088: beq.b $117092
        e.step()?;
        pc = if r.equal()? {
            0x117092_u32
        } else {
            0x11708a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1be_unit::combat_body_block_1be;

mod combat_body_block_1bf_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1bf(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011708a: move.w #$863, $443316.l
        e.step()?;
        let value = 0x863_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x117092_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1bf_unit::combat_body_block_1bf;

mod combat_body_block_1c0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1c0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00117092: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00117098: addi.w #$16e, d0
        e.step()?;
        let source = 0x16e_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011709c: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001170a2: moveq #$33, d0
        e.step()?;
        r.put_data(0, 32, 0x33_u32);
        r.move_flags(0x33_u32, 32);
        // 001170a4: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001170aa: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001170b0: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 001170b2: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001170b8: pea.l $44292c.l
        e.step()?;
        let value = 0x44292c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001170be: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 001170c0: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001170c2: beq.b $1170cc
        e.step()?;
        pc = if r.equal()? {
            0x1170cc_u32
        } else {
            0x1170c4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1c0_unit::combat_body_block_1c0;

mod combat_body_block_1c1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1c1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001170c4: move.w #$e65, $443316.l
        e.step()?;
        let value = 0xe65_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1170cc_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1c1_unit::combat_body_block_1c1;

mod combat_body_block_1c2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1c2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001170cc: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 001170d2: addi.w #$1a8, d0
        e.step()?;
        let source = 0x1a8_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001170d6: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001170dc: moveq #$29, d0
        e.step()?;
        r.put_data(0, 32, 0x29_u32);
        r.move_flags(0x29_u32, 32);
        // 001170de: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001170e4: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001170ea: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 001170ec: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001170f2: pea.l $44292c.l
        e.step()?;
        let value = 0x44292c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001170f8: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 001170fa: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001170fc: beq.b $117106
        e.step()?;
        pc = if r.equal()? {
            0x117106_u32
        } else {
            0x1170fe_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1c2_unit::combat_body_block_1c2;

mod combat_body_block_1c3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1c3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001170fe: move.w #$2075, $443316.l
        e.step()?;
        let value = 0x2075_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x117106_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1c3_unit::combat_body_block_1c3;

mod combat_body_block_1c4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1c4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00117106: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011710c: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 0011710e: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00117114: addi.w #$15c, d1
        e.step()?;
        let source = 0x15c_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 00117118: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0011711a: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00117120: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 00117122: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00117128: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011712e: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00117134: addi.w #$20c, d0
        e.step()?;
        let source = 0x20c_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00117138: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011713e: moveq #$29, d0
        e.step()?;
        r.put_data(0, 32, 0x29_u32);
        r.move_flags(0x29_u32, 32);
        // 00117140: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 00117146: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 0011714c: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 0011714e: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00117154: pea.l $44292c.l
        e.step()?;
        let value = 0x44292c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 0011715a: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 0011715c: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 0011715e: beq.b $117168
        e.step()?;
        pc = if r.equal()? {
            0x117168_u32
        } else {
            0x117160_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1c4_unit::combat_body_block_1c4;

mod combat_body_block_1c5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1c5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00117160: move.w #$1174, $443316.l
        e.step()?;
        let value = 0x1174_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x117168_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1c5_unit::combat_body_block_1c5;

mod combat_body_block_1c6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1c6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00117168: move.w $439582.l, d0
        e.step()?;
        let value_address = 0x439582_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011716e: lsl.w #$5, d0
        e.step()?;
        let count = 0x5_u32;
        let source = r.data(0, 16)?;
        let value = r.shift_left(source, count & 63, 16);
        r.put_data(0, 16, value);
        // 00117170: move.w $40856e.l, d1
        e.step()?;
        let value_address = 0x40856e_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(1, 16, value);
        r.move_flags(value, 16);
        // 00117176: addi.w #$148, d1
        e.step()?;
        let source = 0x148_u32;
        let destination = r.data(1, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0011717a: sub.w d0, d1
        e.step()?;
        let source = r.data(0, 16)?;
        let destination = r.data(1, 16)?;
        let value = r.subtract_flags(destination, source, 16, true);
        r.put_data(1, 16, value);
        // 0011717c: move.w d1, $44292c.l
        e.step()?;
        let value = r.data(1, 16)?;
        let destination_address = 0x44292c_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00117182: moveq #$f, d0
        e.step()?;
        r.put_data(0, 32, 0xf_u32);
        r.move_flags(0xf_u32, 32);
        // 00117184: add.w $44292c.l, d0
        e.step()?;
        let source_address = 0x44292c_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011718a: move.w d0, $442930.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442930_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00117190: move.w $40856c.l, d0
        e.step()?;
        let value_address = 0x40856c_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00117196: addi.w #$20c, d0
        e.step()?;
        let source = 0x20c_u32;
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 0011719a: move.w d0, $44292e.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x44292e_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001171a0: moveq #$29, d0
        e.step()?;
        r.put_data(0, 32, 0x29_u32);
        r.move_flags(0x29_u32, 32);
        // 001171a2: add.w $44292e.l, d0
        e.step()?;
        let source_address = 0x44292e_u32;
        let source = u32::from(e.w(source_address)?);
        let destination = r.data(0, 16)?;
        let value = r.add_flags(destination, source, 16, true);
        r.put_data(0, 16, value);
        // 001171a8: move.w d0, $442932.l
        e.step()?;
        let value = r.data(0, 16)?;
        let destination_address = 0x442932_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001171ae: subq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_sub(source);
        r.put_address(7, value);
        // 001171b0: move.l $4471ee.l, -(a7)
        e.step()?;
        let value_address = 0x4471ee_u32;
        let value = e.l(value_address)?;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001171b6: pea.l $44292c.l
        e.step()?;
        let value = 0x44292c_u32;
        r.offset_address(7, -4)?;
        e.long(r.address(7)?, value)?;
        // 001171bc: MacToolbox 0xa8ad
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa8ad_u32 as u16))?;
        // 001171be: move.b (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = u32::from(e.b(value_address)?);
        r.offset_address(7, 2)?;
        r.put_data(0, 8, value);
        r.move_flags(value, 8);
        // 001171c0: beq.b $1171ca
        e.step()?;
        pc = if r.equal()? {
            0x1171ca_u32
        } else {
            0x1171c2_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1c6_unit::combat_body_block_1c6;

mod combat_body_block_1c7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1c7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001171c2: move.w #$d77, $443316.l
        e.step()?;
        let value = 0xd77_u32;
        let destination_address = 0x443316_u32;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        pc = 0x1171ca_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1c7_unit::combat_body_block_1c7;

mod combat_body_block_1c8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1c8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001171ca: tst.w $443316.l
        e.step()?;
        let value_address = 0x443316_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001171d0: bne.w $115b72
        e.step()?;
        pc = if r.not_equal()? {
            0x115b72_u32
        } else {
            0x1171d4_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1c8_unit::combat_body_block_1c8;

mod combat_body_block_1c9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1c9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001171d4: bsr.l $117be2
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1171da_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x117be2_u32)?;
        // 001171da: move.w #$1e, -(a7)
        e.step()?;
        let value = 0x1e_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001171de: bsr.l $13154c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1171e4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13154c_u32)?;
        // 001171e4: tst.w $438776.l
        e.step()?;
        let value_address = 0x438776_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 001171ea: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001171ec: bne.b $1171f4
        e.step()?;
        pc = if r.not_equal()? {
            0x1171f4_u32
        } else {
            0x1171ee_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1c9_unit::combat_body_block_1c9;

mod combat_body_block_1ca_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1ca(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001171ee: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        // 001171f0: bra.w $117344
        e.step()?;
        pc = 0x117344_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1ca_unit::combat_body_block_1ca;

mod combat_body_block_1cb_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1cb(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001171f4: move.w #$1f, -(a7)
        e.step()?;
        let value = 0x1f_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001171f8: bsr.l $13154c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1171fe_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13154c_u32)?;
        // 001171fe: bsr.l $121524
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x117204_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x121524_u32)?;
        // 00117204: move.w #$20, -(a7)
        e.step()?;
        let value = 0x20_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00117208: bsr.l $13154c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11720e_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13154c_u32)?;
        // 0011720e: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00117214: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00117218: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0011721e: cmpi.w #$2, $40(a0, d0.l)
        e.step()?;
        let source = 0x2_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x40_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00117224: addq.w #$4, a7
        e.step()?;
        let source = 0x4_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 00117226: bge.b $11724a
        e.step()?;
        pc = if r.greater_equal()? {
            0x11724a_u32
        } else {
            0x117228_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1cb_unit::combat_body_block_1cb;

mod combat_body_block_1cc_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1cc(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00117228: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 0011722e: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00117232: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00117238: adda.l d0, a0
        e.step()?;
        let source = r.data(0, 32)?;
        let destination = r.address(0)?;
        let value = destination.wrapping_add(source);
        r.put_address(0, value);
        // 0011723a: tst.b $9dd(a0)
        e.step()?;
        let value_address = r.address(0)?.wrapping_add(0x9dd_u32);
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 0011723e: bne.b $11724a
        e.step()?;
        pc = if r.not_equal()? {
            0x11724a_u32
        } else {
            0x117240_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1cc_unit::combat_body_block_1cc;

mod combat_body_block_1cd_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1cd(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00117240: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00117242: bsr.l $125a3a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x117248_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x125a3a_u32)?;
        // 00117248: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x11724a_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1cd_unit::combat_body_block_1cd;

mod combat_body_block_1ce_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1ce(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011724a: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00117250: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00117254: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 0011725a: cmpi.w #$1, $40(a0, d0.l)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x40_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00117260: blt.b $117282
        e.step()?;
        pc = if r.less()? {
            0x117282_u32
        } else {
            0x117262_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1ce_unit::combat_body_block_1ce;

mod combat_body_block_1cf_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1cf(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00117262: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00117268: mulu.w #$c42, d0
        e.step()?;
        let source = 0xc42_u32;
        let destination = r.data(0, 16)?;
        let value = source.wrapping_mul(destination);
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 0011726c: lea.l $448aa0.l, a0
        e.step()?;
        let value = 0x448aa0_u32;
        r.put_address(0, value);
        // 00117272: cmpi.w #$1, $40(a0, d0.l)
        e.step()?;
        let source = 0x1_u32;
        let destination_address = r
            .address(0)?
            .wrapping_add(0x40_u32)
            .wrapping_add((r.data(0, 32)?).wrapping_mul(1));
        let destination = u32::from(e.w(destination_address)?);
        r.subtract_flags(destination, source, 16, false);
        // 00117278: bne.b $11728e
        e.step()?;
        pc = if r.not_equal()? {
            0x11728e_u32
        } else {
            0x11727a_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1cf_unit::combat_body_block_1cf;

mod combat_body_block_1d0_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1d0(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011727a: tst.w $41f712.l
        e.step()?;
        let value_address = 0x41f712_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 00117280: beq.b $11728e
        e.step()?;
        pc = if r.equal()? {
            0x11728e_u32
        } else {
            0x117282_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1d0_unit::combat_body_block_1d0;

mod combat_body_block_1d1_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1d1(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00117282: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00117284: bsr.l $125a3a
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11728a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x125a3a_u32)?;
        // 0011728a: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0011728c: bra.b $1172aa
        e.step()?;
        pc = 0x1172aa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1d1_unit::combat_body_block_1d1;

mod combat_body_block_1d2_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1d2(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 0011728e: move.w $4435d2.l, -(a7)
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00117294: bsr.l $12946c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11729a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x12946c_u32)?;
        // 0011729a: tst.w d0
        e.step()?;
        let value = r.data(0, 16)?;
        r.move_flags(value, 16);
        // 0011729c: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 0011729e: bne.b $1172aa
        e.step()?;
        pc = if r.not_equal()? {
            0x1172aa_u32
        } else {
            0x1172a0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1d2_unit::combat_body_block_1d2;

mod combat_body_block_1d3_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1d3(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001172a0: clr.w -(a7)
        e.step()?;
        let value = 0;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001172a2: bsr.l $110644
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1172a8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x110644_u32)?;
        // 001172a8: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        pc = 0x1172aa_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1d3_unit::combat_body_block_1d3;

mod combat_body_block_1d4_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1d4(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001172aa: move.w #$21, -(a7)
        e.step()?;
        let value = 0x21_u32;
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001172ae: bsr.l $13154c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1172b4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13154c_u32)?;
        // 001172b4: addq.w #$2, a7
        e.step()?;
        let source = 0x2_u32;
        let destination = r.address(7)?;
        let source = i32::from(source as u16 as i16) as u32;
        let value = destination.wrapping_add(source);
        r.put_address(7, value);
        // 001172b6: bra.w $117338
        e.step()?;
        pc = 0x117338_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1d4_unit::combat_body_block_1d4;

mod combat_body_block_1d5_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1d5(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001172ba: move.l $44720c.l, d0
        e.step()?;
        let value_address = 0x44720c_u32;
        let value = e.l(value_address)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 001172c0: moveq #$18, d1
        e.step()?;
        r.put_data(1, 32, 0x18_u32);
        r.move_flags(0x18_u32, 32);
        // 001172c2: lsr.l d1, d0
        e.step()?;
        let count = r.data(1, 6)?;
        let source = r.data(0, 32)?;
        let value = r.shift_right(source, count & 63, 32, false);
        r.put_data(0, 32, value);
        // 001172c4: subq.l #$1, d0
        e.step()?;
        let source = 0x1_u32;
        let destination = r.data(0, 32)?;
        let value = r.subtract_flags(destination, source, 32, true);
        r.put_data(0, 32, value);
        // 001172c6: bne.b $117338
        e.step()?;
        pc = if r.not_equal()? {
            0x117338_u32
        } else {
            0x1172c8_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1d5_unit::combat_body_block_1d5;

mod combat_body_block_1d6_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1d6(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001172c8: tst.b $4427b2.l
        e.step()?;
        let value_address = 0x4427b2_u32;
        let value = u32::from(e.b(value_address)?);
        r.move_flags(value, 8);
        // 001172ce: beq.b $117330
        e.step()?;
        pc = if r.equal()? {
            0x117330_u32
        } else {
            0x1172d0_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1d6_unit::combat_body_block_1d6;

mod combat_body_block_1d7_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1d7(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 001172d0: bsr.l $13e13c
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1172d6_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13e13c_u32)?;
        // 001172d6: clr.b $4427b2.l
        e.step()?;
        let value = 0;
        let destination_address = 0x4427b2_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        // 001172dc: move.l #$10000, -(a7)
        e.step()?;
        let value = 0x10000_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 001172e2: bsr.l $13bb20
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1172e8_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x13bb20_u32)?;
        // 001172e8: move.w $4435d2.l, -(a7)
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 001172ee: bsr.l $11f1a8
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x1172f4_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x11f1a8_u32)?;
        // 001172f4: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 001172fa: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00117300: move.w $2(a0, d0.w * 4), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x2_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00117304: lea.l $44709a.l, a0
        e.step()?;
        let value = 0x44709a_u32;
        r.put_address(0, value);
        // 0011730a: move.w $4435d2.l, d0
        e.step()?;
        let value_address = 0x4435d2_u32;
        let value = u32::from(e.w(value_address)?);
        r.put_data(0, 16, value);
        r.move_flags(value, 16);
        // 00117310: move.w (a0, d0.w * 4), -(a7)
        e.step()?;
        let value_address = r
            .address(0)?
            .wrapping_add(0x0_u32)
            .wrapping_add((i32::from(r.data(0, 16)? as u16 as i16) as u32).wrapping_mul(4));
        let value = u32::from(e.w(value_address)?);
        r.offset_address(7, -2)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 16, value)?;
        r.move_flags(value, 16);
        // 00117314: bsr.l $10e6b6
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x11731a_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x10e6b6_u32)?;
        // 0011731a: bsr.l $16043e
        e.step()?;
        e.long(r.address(7)?.wrapping_sub(4), 0x117320_u32)?;
        e.long(r.address(7)?.wrapping_sub(8), r.address(6)?)?;
        child(e, r, 0x16043e_u32)?;
        // 00117320: move.l #$ffff, -(a7)
        e.step()?;
        let value = 0xffff_u32;
        r.offset_address(7, -4)?;
        let destination_address = r.address(7)?;
        r.write_memory(e, destination_address, 32, value)?;
        r.move_flags(value, 32);
        // 00117326: move.l (a7)+, d0
        e.step()?;
        let value_address = r.address(7)?;
        let value = e.l(value_address)?;
        r.offset_address(7, 4)?;
        r.put_data(0, 32, value);
        r.move_flags(value, 32);
        // 00117328: MacToolbox 0xa032
        e.step()?;
        imported(e, r, NativeCall::Toolbox(0xa032_u32 as u16))?;
        // 0011732a: lea.l $a(a7), a7
        e.step()?;
        let value = r.address(7)?.wrapping_add(0xa_u32);
        r.put_address(7, value);
        // 0011732e: bra.b $117338
        e.step()?;
        pc = 0x117338_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1d7_unit::combat_body_block_1d7;

mod combat_body_block_1d8_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1d8(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00117330: move.b #$1, $4427b2.l
        e.step()?;
        let value = 0x1_u32;
        let destination_address = 0x4427b2_u32;
        r.write_memory(e, destination_address, 8, value)?;
        r.move_flags(value, 8);
        pc = 0x117338_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1d8_unit::combat_body_block_1d8;

mod combat_body_block_1d9_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1d9(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00117338: tst.w $438776.l
        e.step()?;
        let value_address = 0x438776_u32;
        let value = u32::from(e.w(value_address)?);
        r.move_flags(value, 16);
        // 0011733e: bne.w $114548
        e.step()?;
        pc = if r.not_equal()? {
            0x114548_u32
        } else {
            0x117342_u32
        };

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1d9_unit::combat_body_block_1d9;

mod combat_body_block_1da_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1da(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00117342: moveq #$0, d0
        e.step()?;
        r.put_data(0, 32, 0x0_u32);
        r.move_flags(0x0_u32, 32);
        pc = 0x117344_u32;

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1da_unit::combat_body_block_1da;

mod combat_body_block_1db_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1db(
        e: &mut Execution<'_>,
        r: &mut centurions_core::recovered_inventory_interaction::ItemsCpu,
        frame: u32,
        mut pc: u32,
    ) -> Result<std::ops::ControlFlow<(), u32>> {
        // 00117344: movem.l (a7)+, d3-d7/a2
        e.step()?;
        r.restore_register(e, 3)?;
        r.restore_register(e, 4)?;
        r.restore_register(e, 5)?;
        r.restore_register(e, 6)?;
        r.restore_register(e, 7)?;
        r.restore_register(e, 10)?;
        // 00117348: unlk a6
        e.step()?;
        r.put_address(7, r.address(6)?);
        r.restore_register(e, 14)?;
        // 0011734a: rts
        e.step()?;
        return finish(e, r).map(|_| std::ops::ControlFlow::Break(()));

        Ok(std::ops::ControlFlow::Continue(pc))
    }
}
use combat_body_block_1db_unit::combat_body_block_1db;

mod combat_body_block_1dc_unit {
    use super::*;
    #[allow(unused_variables, unused_mut, unused_assignments, unreachable_code)]
    #[inline(never)]
    pub(super) fn combat_body_block_1dc(
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
use combat_body_block_1dc_unit::combat_body_block_1dc;
