#![allow(non_camel_case_types, dead_code)]

macro_rules! enum_from_pairs {
    ($name:ident, $(($num:expr, $s:ident, $str:expr, $parent:ident)),*) => {
        #[derive(PartialEq, Debug)]
        pub enum $name {
            $( $s = $num, )*
        }

        impl $name {
            fn from_num(num: u16) -> Option<Self> {
                match num {
                    $( $num => Some($name::$s), )*
                    _ => None
                }
            }

            fn as_str(&self) -> &'static str {
                match self {
                    $( $name::$s => $str, )*
                }
            }

            fn canonical(&self) -> Self {
                match self {
                    $( $name::$s => $name::$parent, )*
                }
            }
        }
    };
}

enum_from_pairs!(
    Aarch64Reg,
    (1, Ffr, "ffr", Ffr),
    (2, Fp, "fp", Fp),
    (3, Lr, "lr", Lr),
    (4, Nzcv, "nzcv", Nzcv),
    (5, Sp, "sp", Sp),
    (6, Vg, "vg", Vg),
    (7, Wsp, "wsp", Wsp),
    (8, Wzr, "wzr", Wzr),
    (9, Xzr, "xzr", Xzr),
    (10, Za, "za", Za),
    (11, B0, "b0", B0),
    (12, B1, "b1", B1),
    (13, B2, "b2", B2),
    (14, B3, "b3", B3),
    (15, B4, "b4", B4),
    (16, B5, "b5", B5),
    (17, B6, "b6", B6),
    (18, B7, "b7", B7),
    (19, B8, "b8", B8),
    (20, B9, "b9", B9),
    (21, B10, "b10", B10),
    (22, B11, "b11", B11),
    (23, B12, "b12", B12),
    (24, B13, "b13", B13),
    (25, B14, "b14", B14),
    (26, B15, "b15", B15),
    (27, B16, "b16", B16),
    (28, B17, "b17", B17),
    (29, B18, "b18", B18),
    (30, B19, "b19", B19),
    (31, B20, "b20", B20),
    (32, B21, "b21", B21),
    (33, B22, "b22", B22),
    (34, B23, "b23", B23),
    (35, B24, "b24", B24),
    (36, B25, "b25", B25),
    (37, B26, "b26", B26),
    (38, B27, "b27", B27),
    (39, B28, "b28", B28),
    (40, B29, "b29", B29),
    (41, B30, "b30", B30),
    (42, B31, "b31", B31),
    (43, D0, "d0", D0),
    (44, D1, "d1", D1),
    (45, D2, "d2", D2),
    (46, D3, "d3", D3),
    (47, D4, "d4", D4),
    (48, D5, "d5", D5),
    (49, D6, "d6", D6),
    (50, D7, "d7", D7),
    (51, D8, "d8", D8),
    (52, D9, "d9", D9),
    (53, D10, "d10", D10),
    (54, D11, "d11", D11),
    (55, D12, "d12", D12),
    (56, D13, "d13", D13),
    (57, D14, "d14", D14),
    (58, D15, "d15", D15),
    (59, D16, "d16", D16),
    (60, D17, "d17", D17),
    (61, D18, "d18", D18),
    (62, D19, "d19", D19),
    (63, D20, "d20", D20),
    (64, D21, "d21", D21),
    (65, D22, "d22", D22),
    (66, D23, "d23", D23),
    (67, D24, "d24", D24),
    (68, D25, "d25", D25),
    (69, D26, "d26", D26),
    (70, D27, "d27", D27),
    (71, D28, "d28", D28),
    (72, D29, "d29", D29),
    (73, D30, "d30", D30),
    (74, D31, "d31", D31),
    (75, H0, "h0", H0),
    (76, H1, "h1", H1),
    (77, H2, "h2", H2),
    (78, H3, "h3", H3),
    (79, H4, "h4", H4),
    (80, H5, "h5", H5),
    (81, H6, "h6", H6),
    (82, H7, "h7", H7),
    (83, H8, "h8", H8),
    (84, H9, "h9", H9),
    (85, H10, "h10", H10),
    (86, H11, "h11", H11),
    (87, H12, "h12", H12),
    (88, H13, "h13", H13),
    (89, H14, "h14", H14),
    (90, H15, "h15", H15),
    (91, H16, "h16", H16),
    (92, H17, "h17", H17),
    (93, H18, "h18", H18),
    (94, H19, "h19", H19),
    (95, H20, "h20", H20),
    (96, H21, "h21", H21),
    (97, H22, "h22", H22),
    (98, H23, "h23", H23),
    (99, H24, "h24", H24),
    (100, H25, "h25", H25),
    (101, H26, "h26", H26),
    (102, H27, "h27", H27),
    (103, H28, "h28", H28),
    (104, H29, "h29", H29),
    (105, H30, "h30", H30),
    (106, H31, "h31", H31),
    (107, P0, "p0", P0),
    (108, P1, "p1", P1),
    (109, P2, "p2", P2),
    (110, P3, "p3", P3),
    (111, P4, "p4", P4),
    (112, P5, "p5", P5),
    (113, P6, "p6", P6),
    (114, P7, "p7", P7),
    (115, P8, "p8", P8),
    (116, P9, "p9", P9),
    (117, P10, "p10", P10),
    (118, P11, "p11", P11),
    (119, P12, "p12", P12),
    (120, P13, "p13", P13),
    (121, P14, "p14", P14),
    (122, P15, "p15", P15),
    (123, Q0, "q0", Q0),
    (124, Q1, "q1", Q1),
    (125, Q2, "q2", Q2),
    (126, Q3, "q3", Q3),
    (127, Q4, "q4", Q4),
    (128, Q5, "q5", Q5),
    (129, Q6, "q6", Q6),
    (130, Q7, "q7", Q7),
    (131, Q8, "q8", Q8),
    (132, Q9, "q9", Q9),
    (133, Q10, "q10", Q10),
    (134, Q11, "q11", Q11),
    (135, Q12, "q12", Q12),
    (136, Q13, "q13", Q13),
    (137, Q14, "q14", Q14),
    (138, Q15, "q15", Q15),
    (139, Q16, "q16", Q16),
    (140, Q17, "q17", Q17),
    (141, Q18, "q18", Q18),
    (142, Q19, "q19", Q19),
    (143, Q20, "q20", Q20),
    (144, Q21, "q21", Q21),
    (145, Q22, "q22", Q22),
    (146, Q23, "q23", Q23),
    (147, Q24, "q24", Q24),
    (148, Q25, "q25", Q25),
    (149, Q26, "q26", Q26),
    (150, Q27, "q27", Q27),
    (151, Q28, "q28", Q28),
    (152, Q29, "q29", Q29),
    (153, Q30, "q30", Q30),
    (154, Q31, "q31", Q31),
    (155, S0, "s0", S0),
    (156, S1, "s1", S1),
    (157, S2, "s2", S2),
    (158, S3, "s3", S3),
    (159, S4, "s4", S4),
    (160, S5, "s5", S5),
    (161, S6, "s6", S6),
    (162, S7, "s7", S7),
    (163, S8, "s8", S8),
    (164, S9, "s9", S9),
    (165, S10, "s10", S10),
    (166, S11, "s11", S11),
    (167, S12, "s12", S12),
    (168, S13, "s13", S13),
    (169, S14, "s14", S14),
    (170, S15, "s15", S15),
    (171, S16, "s16", S16),
    (172, S17, "s17", S17),
    (173, S18, "s18", S18),
    (174, S19, "s19", S19),
    (175, S20, "s20", S20),
    (176, S21, "s21", S21),
    (177, S22, "s22", S22),
    (178, S23, "s23", S23),
    (179, S24, "s24", S24),
    (180, S25, "s25", S25),
    (181, S26, "s26", S26),
    (182, S27, "s27", S27),
    (183, S28, "s28", S28),
    (184, S29, "s29", S29),
    (185, S30, "s30", S30),
    (186, S31, "s31", S31),
    (187, W0, "w0", W0),
    (188, W1, "w1", W1),
    (189, W2, "w2", W2),
    (190, W3, "w3", W3),
    (191, W4, "w4", W4),
    (192, W5, "w5", W5),
    (193, W6, "w6", W6),
    (194, W7, "w7", W7),
    (195, W8, "w8", W8),
    (196, W9, "w9", W9),
    (197, W10, "w10", W10),
    (198, W11, "w11", W11),
    (199, W12, "w12", W12),
    (200, W13, "w13", W13),
    (201, W14, "w14", W14),
    (202, W15, "w15", W15),
    (203, W16, "w16", W16),
    (204, W17, "w17", W17),
    (205, W18, "w18", W18),
    (206, W19, "w19", W19),
    (207, W20, "w20", W20),
    (208, W21, "w21", W21),
    (209, W22, "w22", W22),
    (210, W23, "w23", W23),
    (211, W24, "w24", W24),
    (212, W25, "w25", W25),
    (213, W26, "w26", W26),
    (214, W27, "w27", W27),
    (215, W28, "w28", W28),
    (216, W29, "w29", W29),
    (217, W30, "w30", W30),
    (218, X0, "x0", X0),
    (219, X1, "x1", X1),
    (220, X2, "x2", X2),
    (221, X3, "x3", X3),
    (222, X4, "x4", X4),
    (223, X5, "x5", X5),
    (224, X6, "x6", X6),
    (225, X7, "x7", X7),
    (226, X8, "x8", X8),
    (227, X9, "x9", X9),
    (228, X10, "x10", X10),
    (229, X11, "x11", X11),
    (230, X12, "x12", X12),
    (231, X13, "x13", X13),
    (232, X14, "x14", X14),
    (233, X15, "x15", X15),
    (234, X16, "x16", X16),
    (235, X17, "x17", X17),
    (236, X18, "x18", X18),
    (237, X19, "x19", X19),
    (238, X20, "x20", X20),
    (239, X21, "x21", X21),
    (240, X22, "x22", X22),
    (241, X23, "x23", X23),
    (242, X24, "x24", X24),
    (243, X25, "x25", X25),
    (244, X26, "x26", X26),
    (245, X27, "x27", X27),
    (246, X28, "x28", X28),
    (247, Z0, "z0", Z0),
    (248, Z1, "z1", Z1),
    (249, Z2, "z2", Z2),
    (250, Z3, "z3", Z3),
    (251, Z4, "z4", Z4),
    (252, Z5, "z5", Z5),
    (253, Z6, "z6", Z6),
    (254, Z7, "z7", Z7),
    (255, Z8, "z8", Z8),
    (256, Z9, "z9", Z9),
    (257, Z10, "z10", Z10),
    (258, Z11, "z11", Z11),
    (259, Z12, "z12", Z12),
    (260, Z13, "z13", Z13),
    (261, Z14, "z14", Z14),
    (262, Z15, "z15", Z15),
    (263, Z16, "z16", Z16),
    (264, Z17, "z17", Z17),
    (265, Z18, "z18", Z18),
    (266, Z19, "z19", Z19),
    (267, Z20, "z20", Z20),
    (268, Z21, "z21", Z21),
    (269, Z22, "z22", Z22),
    (270, Z23, "z23", Z23),
    (271, Z24, "z24", Z24),
    (272, Z25, "z25", Z25),
    (273, Z26, "z26", Z26),
    (274, Z27, "z27", Z27),
    (275, Z28, "z28", Z28),
    (276, Z29, "z29", Z29),
    (277, Z30, "z30", Z30),
    (278, Z31, "z31", Z31),
    (279, Zab0, "zab0", Zab0),
    (280, Zad0, "zad0", Zad0),
    (281, Zad1, "zad1", Zad1),
    (282, Zad2, "zad2", Zad2),
    (283, Zad3, "zad3", Zad3),
    (284, Zad4, "zad4", Zad4),
    (285, Zad5, "zad5", Zad5),
    (286, Zad6, "zad6", Zad6),
    (287, Zad7, "zad7", Zad7),
    (288, Zah0, "zah0", Zah0),
    (289, Zah1, "zah1", Zah1),
    (290, Zaq0, "zaq0", Zaq0),
    (291, Zaq1, "zaq1", Zaq1),
    (292, Zaq2, "zaq2", Zaq2),
    (293, Zaq3, "zaq3", Zaq3),
    (294, Zaq4, "zaq4", Zaq4),
    (295, Zaq5, "zaq5", Zaq5),
    (296, Zaq6, "zaq6", Zaq6),
    (297, Zaq7, "zaq7", Zaq7),
    (298, Zaq8, "zaq8", Zaq8),
    (299, Zaq9, "zaq9", Zaq9),
    (300, Zaq10, "zaq10", Zaq10),
    (301, Zaq11, "zaq11", Zaq11),
    (302, Zaq12, "zaq12", Zaq12),
    (303, Zaq13, "zaq13", Zaq13),
    (304, Zaq14, "zaq14", Zaq14),
    (305, Zaq15, "zaq15", Zaq15),
    (306, Zas0, "zas0", Zas0),
    (307, Zas1, "zas1", Zas1),
    (308, Zas2, "zas2", Zas2),
    (309, Zas3, "zas3", Zas3),
    (310, V0, "v0", V0),
    (311, V1, "v1", V1),
    (312, V2, "v2", V2),
    (313, V3, "v3", V3),
    (314, V4, "v4", V4),
    (315, V5, "v5", V5),
    (316, V6, "v6", V6),
    (317, V7, "v7", V7),
    (318, V8, "v8", V8),
    (319, V9, "v9", V9),
    (320, V10, "v10", V10),
    (321, V11, "v11", V11),
    (322, V12, "v12", V12),
    (323, V13, "v13", V13),
    (324, V14, "v14", V14),
    (325, V15, "v15", V15),
    (326, V16, "v16", V16),
    (327, V17, "v17", V17),
    (328, V18, "v18", V18),
    (329, V19, "v19", V19),
    (330, V20, "v20", V20),
    (331, V21, "v21", V21),
    (332, V22, "v22", V22),
    (333, V23, "v23", V23),
    (334, V24, "v24", V24),
    (335, V25, "v25", V25),
    (336, V26, "v26", V26),
    (337, V27, "v27", V27),
    (338, V28, "v28", V28),
    (339, V29, "v29", V29),
    (340, V30, "v30", V30),
    (341, V31, "v31", V31)
);

pub enum Reg {
    Aarch64Reg(Aarch64Reg),
    // X64Reg(X64Reg),
}

// impl Reg {
//     pub fn from_num(arch: Arch, num: u8) -> Option<Self> {
//         Some(match arch {
//             Arch::ARM64 => Reg::Aarch64Reg(Aarch64Reg::from_num(num)?),
//             Arch::X86_64 => Reg::X64Reg(X64Reg::from_num(num)?),
//         })
//     }
// }

#[cfg(test)]
mod tests {
    use capstone::RegId;
    use convert_case::Casing;

    use super::Aarch64Reg;
    use crate::{arch::Arch, dis::regs::Reg};

    #[test]
    fn aarch64_canon() {
        // assert_eq!(Aarch64Reg::W0.canonical(), Aarch64Reg::X0);
        // assert_eq!(Aarch64Reg::X5.canonical(), Aarch64Reg::X5);
        // assert_eq!(Aarch64Reg::SP.canonical(), Aarch64Reg::SP);
        // assert_eq!(Aarch64Reg::XZR.canonical(), Aarch64Reg::XZR);
        // assert_eq!(Aarch64Reg::X30.canonical(), Aarch64Reg::X30);
        // assert_eq!(Aarch64Reg::W30.canonical(), Aarch64Reg::X30);
    }

    #[test]
    fn aarch64_reg_names() {
        let arch = Arch::ARM64;
        let cs = arch.make_capstone().unwrap();

        for i in 0..=u16::MAX {
            let i = RegId(i);

            let cs_name = cs.reg_name(i);

            // let dis_name = None;

            // if cs_name != dis_name {
            //     panic!("{cs_name:?} != {dis_name:?}")
            // }

            // cargo test aarch64_reg_names -- --nocapture
            // if let Some(name) = cs_name {
            //     println!(
            //         "({}, {}, \"{}\", {}),",
            //         i.0,
            //         name.to_case(convert_case::Case::Pascal),
            //         name,
            //         name.to_case(convert_case::Case::Pascal),
            //     );
            // }
        }
    }
}
