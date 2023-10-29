#![allow(non_camel_case_types, dead_code)]

use crate::arch::Arch;

macro_rules! enum_from_pairs {
    ($name:ident, $(($num:expr, $s:ident, $str:expr, $parent:ident, $idx:expr)),*) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum $name {
            $( $s = $num, )*
        }

        impl $name {
            pub fn from_num(num: u16) -> Option<Self> {
                match num {
                    $( $num => Some($name::$s), )*
                    _ => None
                }
            }

            pub fn as_str(&self) -> &'static str {
                match self {
                    $( $name::$s => $str, )*
                }
            }

            pub fn canonical(&self) -> Self {
                match self {
                    $( $name::$s => $name::$parent, )*
                }
            }

            pub fn idx(&self) -> Option<usize> {
                match self {
                    $( $name::$s => $idx, )*
                }
            }

            pub fn from_idx(i: usize) -> Option<Self> {
                $(
                    if Some(i) == $idx {
                        return Some($name::$s);
                    }
                )*
                None
            }
        }
    };
}

enum_from_pairs!(
    Aarch64Reg,
    (1, Ffr, "ffr", Ffr, None),
    (2, Fp, "fp", Fp, Some(29)),
    (3, Lr, "lr", Lr, Some(30)),
    (4, Nzcv, "nzcv", Nzcv, None),
    (5, Sp, "sp", Sp, Some(31)),
    (6, Vg, "vg", Vg, None),
    (7, Wsp, "wsp", Wsp, None),
    (8, Wzr, "wzr", Wzr, None),
    (9, Xzr, "xzr", Xzr, None),
    (10, Za, "za", Za, None),
    (11, B0, "b0", B0, None),
    (12, B1, "b1", B1, None),
    (13, B2, "b2", B2, None),
    (14, B3, "b3", B3, None),
    (15, B4, "b4", B4, None),
    (16, B5, "b5", B5, None),
    (17, B6, "b6", B6, None),
    (18, B7, "b7", B7, None),
    (19, B8, "b8", B8, None),
    (20, B9, "b9", B9, None),
    (21, B10, "b10", B10, None),
    (22, B11, "b11", B11, None),
    (23, B12, "b12", B12, None),
    (24, B13, "b13", B13, None),
    (25, B14, "b14", B14, None),
    (26, B15, "b15", B15, None),
    (27, B16, "b16", B16, None),
    (28, B17, "b17", B17, None),
    (29, B18, "b18", B18, None),
    (30, B19, "b19", B19, None),
    (31, B20, "b20", B20, None),
    (32, B21, "b21", B21, None),
    (33, B22, "b22", B22, None),
    (34, B23, "b23", B23, None),
    (35, B24, "b24", B24, None),
    (36, B25, "b25", B25, None),
    (37, B26, "b26", B26, None),
    (38, B27, "b27", B27, None),
    (39, B28, "b28", B28, None),
    (40, B29, "b29", B29, None),
    (41, B30, "b30", B30, None),
    (42, B31, "b31", B31, None),
    (43, D0, "d0", D0, None),
    (44, D1, "d1", D1, None),
    (45, D2, "d2", D2, None),
    (46, D3, "d3", D3, None),
    (47, D4, "d4", D4, None),
    (48, D5, "d5", D5, None),
    (49, D6, "d6", D6, None),
    (50, D7, "d7", D7, None),
    (51, D8, "d8", D8, None),
    (52, D9, "d9", D9, None),
    (53, D10, "d10", D10, None),
    (54, D11, "d11", D11, None),
    (55, D12, "d12", D12, None),
    (56, D13, "d13", D13, None),
    (57, D14, "d14", D14, None),
    (58, D15, "d15", D15, None),
    (59, D16, "d16", D16, None),
    (60, D17, "d17", D17, None),
    (61, D18, "d18", D18, None),
    (62, D19, "d19", D19, None),
    (63, D20, "d20", D20, None),
    (64, D21, "d21", D21, None),
    (65, D22, "d22", D22, None),
    (66, D23, "d23", D23, None),
    (67, D24, "d24", D24, None),
    (68, D25, "d25", D25, None),
    (69, D26, "d26", D26, None),
    (70, D27, "d27", D27, None),
    (71, D28, "d28", D28, None),
    (72, D29, "d29", D29, None),
    (73, D30, "d30", D30, None),
    (74, D31, "d31", D31, None),
    (75, H0, "h0", H0, None),
    (76, H1, "h1", H1, None),
    (77, H2, "h2", H2, None),
    (78, H3, "h3", H3, None),
    (79, H4, "h4", H4, None),
    (80, H5, "h5", H5, None),
    (81, H6, "h6", H6, None),
    (82, H7, "h7", H7, None),
    (83, H8, "h8", H8, None),
    (84, H9, "h9", H9, None),
    (85, H10, "h10", H10, None),
    (86, H11, "h11", H11, None),
    (87, H12, "h12", H12, None),
    (88, H13, "h13", H13, None),
    (89, H14, "h14", H14, None),
    (90, H15, "h15", H15, None),
    (91, H16, "h16", H16, None),
    (92, H17, "h17", H17, None),
    (93, H18, "h18", H18, None),
    (94, H19, "h19", H19, None),
    (95, H20, "h20", H20, None),
    (96, H21, "h21", H21, None),
    (97, H22, "h22", H22, None),
    (98, H23, "h23", H23, None),
    (99, H24, "h24", H24, None),
    (100, H25, "h25", H25, None),
    (101, H26, "h26", H26, None),
    (102, H27, "h27", H27, None),
    (103, H28, "h28", H28, None),
    (104, H29, "h29", H29, None),
    (105, H30, "h30", H30, None),
    (106, H31, "h31", H31, None),
    (107, P0, "p0", P0, None),
    (108, P1, "p1", P1, None),
    (109, P2, "p2", P2, None),
    (110, P3, "p3", P3, None),
    (111, P4, "p4", P4, None),
    (112, P5, "p5", P5, None),
    (113, P6, "p6", P6, None),
    (114, P7, "p7", P7, None),
    (115, P8, "p8", P8, None),
    (116, P9, "p9", P9, None),
    (117, P10, "p10", P10, None),
    (118, P11, "p11", P11, None),
    (119, P12, "p12", P12, None),
    (120, P13, "p13", P13, None),
    (121, P14, "p14", P14, None),
    (122, P15, "p15", P15, None),
    (123, Q0, "q0", Q0, None),
    (124, Q1, "q1", Q1, None),
    (125, Q2, "q2", Q2, None),
    (126, Q3, "q3", Q3, None),
    (127, Q4, "q4", Q4, None),
    (128, Q5, "q5", Q5, None),
    (129, Q6, "q6", Q6, None),
    (130, Q7, "q7", Q7, None),
    (131, Q8, "q8", Q8, None),
    (132, Q9, "q9", Q9, None),
    (133, Q10, "q10", Q10, None),
    (134, Q11, "q11", Q11, None),
    (135, Q12, "q12", Q12, None),
    (136, Q13, "q13", Q13, None),
    (137, Q14, "q14", Q14, None),
    (138, Q15, "q15", Q15, None),
    (139, Q16, "q16", Q16, None),
    (140, Q17, "q17", Q17, None),
    (141, Q18, "q18", Q18, None),
    (142, Q19, "q19", Q19, None),
    (143, Q20, "q20", Q20, None),
    (144, Q21, "q21", Q21, None),
    (145, Q22, "q22", Q22, None),
    (146, Q23, "q23", Q23, None),
    (147, Q24, "q24", Q24, None),
    (148, Q25, "q25", Q25, None),
    (149, Q26, "q26", Q26, None),
    (150, Q27, "q27", Q27, None),
    (151, Q28, "q28", Q28, None),
    (152, Q29, "q29", Q29, None),
    (153, Q30, "q30", Q30, None),
    (154, Q31, "q31", Q31, None),
    (155, S0, "s0", S0, None),
    (156, S1, "s1", S1, None),
    (157, S2, "s2", S2, None),
    (158, S3, "s3", S3, None),
    (159, S4, "s4", S4, None),
    (160, S5, "s5", S5, None),
    (161, S6, "s6", S6, None),
    (162, S7, "s7", S7, None),
    (163, S8, "s8", S8, None),
    (164, S9, "s9", S9, None),
    (165, S10, "s10", S10, None),
    (166, S11, "s11", S11, None),
    (167, S12, "s12", S12, None),
    (168, S13, "s13", S13, None),
    (169, S14, "s14", S14, None),
    (170, S15, "s15", S15, None),
    (171, S16, "s16", S16, None),
    (172, S17, "s17", S17, None),
    (173, S18, "s18", S18, None),
    (174, S19, "s19", S19, None),
    (175, S20, "s20", S20, None),
    (176, S21, "s21", S21, None),
    (177, S22, "s22", S22, None),
    (178, S23, "s23", S23, None),
    (179, S24, "s24", S24, None),
    (180, S25, "s25", S25, None),
    (181, S26, "s26", S26, None),
    (182, S27, "s27", S27, None),
    (183, S28, "s28", S28, None),
    (184, S29, "s29", S29, None),
    (185, S30, "s30", S30, None),
    (186, S31, "s31", S31, None),
    (187, W0, "w0", X0, None),
    (188, W1, "w1", X1, None),
    (189, W2, "w2", X2, None),
    (190, W3, "w3", X3, None),
    (191, W4, "w4", X4, None),
    (192, W5, "w5", X5, None),
    (193, W6, "w6", X6, None),
    (194, W7, "w7", X7, None),
    (195, W8, "w8", X8, None),
    (196, W9, "w9", X9, None),
    (197, W10, "w10", X10, None),
    (198, W11, "w11", X11, None),
    (199, W12, "w12", X12, None),
    (200, W13, "w13", X13, None),
    (201, W14, "w14", X14, None),
    (202, W15, "w15", X15, None),
    (203, W16, "w16", X16, None),
    (204, W17, "w17", X17, None),
    (205, W18, "w18", X18, None),
    (206, W19, "w19", X19, None),
    (207, W20, "w20", X20, None),
    (208, W21, "w21", X21, None),
    (209, W22, "w22", X22, None),
    (210, W23, "w23", X23, None),
    (211, W24, "w24", X24, None),
    (212, W25, "w25", X25, None),
    (213, W26, "w26", X26, None),
    (214, W27, "w27", X27, None),
    (215, W28, "w28", X28, None),
    (216, W29, "w29", Fp, None), // i think these two make sense :)
    (217, W30, "w30", Lr, None),
    (218, X0, "x0", X0, Some(0)),
    (219, X1, "x1", X1, Some(1)),
    (220, X2, "x2", X2, Some(2)),
    (221, X3, "x3", X3, Some(3)),
    (222, X4, "x4", X4, Some(4)),
    (223, X5, "x5", X5, Some(5)),
    (224, X6, "x6", X6, Some(6)),
    (225, X7, "x7", X7, Some(7)),
    (226, X8, "x8", X8, Some(8)),
    (227, X9, "x9", X9, Some(9)),
    (228, X10, "x10", X10, Some(10)),
    (229, X11, "x11", X11, Some(11)),
    (230, X12, "x12", X12, Some(12)),
    (231, X13, "x13", X13, Some(13)),
    (232, X14, "x14", X14, Some(14)),
    (233, X15, "x15", X15, Some(15)),
    (234, X16, "x16", X16, Some(16)),
    (235, X17, "x17", X17, Some(17)),
    (236, X18, "x18", X18, Some(18)),
    (237, X19, "x19", X19, Some(19)),
    (238, X20, "x20", X20, Some(20)),
    (239, X21, "x21", X21, Some(21)),
    (240, X22, "x22", X22, Some(22)),
    (241, X23, "x23", X23, Some(23)),
    (242, X24, "x24", X24, Some(24)),
    (243, X25, "x25", X25, Some(25)),
    (244, X26, "x26", X26, Some(26)),
    (245, X27, "x27", X27, Some(27)),
    (246, X28, "x28", X28, Some(28)),
    (247, Z0, "z0", Z0, None),
    (248, Z1, "z1", Z1, None),
    (249, Z2, "z2", Z2, None),
    (250, Z3, "z3", Z3, None),
    (251, Z4, "z4", Z4, None),
    (252, Z5, "z5", Z5, None),
    (253, Z6, "z6", Z6, None),
    (254, Z7, "z7", Z7, None),
    (255, Z8, "z8", Z8, None),
    (256, Z9, "z9", Z9, None),
    (257, Z10, "z10", Z10, None),
    (258, Z11, "z11", Z11, None),
    (259, Z12, "z12", Z12, None),
    (260, Z13, "z13", Z13, None),
    (261, Z14, "z14", Z14, None),
    (262, Z15, "z15", Z15, None),
    (263, Z16, "z16", Z16, None),
    (264, Z17, "z17", Z17, None),
    (265, Z18, "z18", Z18, None),
    (266, Z19, "z19", Z19, None),
    (267, Z20, "z20", Z20, None),
    (268, Z21, "z21", Z21, None),
    (269, Z22, "z22", Z22, None),
    (270, Z23, "z23", Z23, None),
    (271, Z24, "z24", Z24, None),
    (272, Z25, "z25", Z25, None),
    (273, Z26, "z26", Z26, None),
    (274, Z27, "z27", Z27, None),
    (275, Z28, "z28", Z28, None),
    (276, Z29, "z29", Z29, None),
    (277, Z30, "z30", Z30, None),
    (278, Z31, "z31", Z31, None),
    (279, Zab0, "zab0", Zab0, None),
    (280, Zad0, "zad0", Zad0, None),
    (281, Zad1, "zad1", Zad1, None),
    (282, Zad2, "zad2", Zad2, None),
    (283, Zad3, "zad3", Zad3, None),
    (284, Zad4, "zad4", Zad4, None),
    (285, Zad5, "zad5", Zad5, None),
    (286, Zad6, "zad6", Zad6, None),
    (287, Zad7, "zad7", Zad7, None),
    (288, Zah0, "zah0", Zah0, None),
    (289, Zah1, "zah1", Zah1, None),
    (290, Zaq0, "zaq0", Zaq0, None),
    (291, Zaq1, "zaq1", Zaq1, None),
    (292, Zaq2, "zaq2", Zaq2, None),
    (293, Zaq3, "zaq3", Zaq3, None),
    (294, Zaq4, "zaq4", Zaq4, None),
    (295, Zaq5, "zaq5", Zaq5, None),
    (296, Zaq6, "zaq6", Zaq6, None),
    (297, Zaq7, "zaq7", Zaq7, None),
    (298, Zaq8, "zaq8", Zaq8, None),
    (299, Zaq9, "zaq9", Zaq9, None),
    (300, Zaq10, "zaq10", Zaq10, None),
    (301, Zaq11, "zaq11", Zaq11, None),
    (302, Zaq12, "zaq12", Zaq12, None),
    (303, Zaq13, "zaq13", Zaq13, None),
    (304, Zaq14, "zaq14", Zaq14, None),
    (305, Zaq15, "zaq15", Zaq15, None),
    (306, Zas0, "zas0", Zas0, None),
    (307, Zas1, "zas1", Zas1, None),
    (308, Zas2, "zas2", Zas2, None),
    (309, Zas3, "zas3", Zas3, None),
    (310, V0, "v0", V0, None),
    (311, V1, "v1", V1, None),
    (312, V2, "v2", V2, None),
    (313, V3, "v3", V3, None),
    (314, V4, "v4", V4, None),
    (315, V5, "v5", V5, None),
    (316, V6, "v6", V6, None),
    (317, V7, "v7", V7, None),
    (318, V8, "v8", V8, None),
    (319, V9, "v9", V9, None),
    (320, V10, "v10", V10, None),
    (321, V11, "v11", V11, None),
    (322, V12, "v12", V12, None),
    (323, V13, "v13", V13, None),
    (324, V14, "v14", V14, None),
    (325, V15, "v15", V15, None),
    (326, V16, "v16", V16, None),
    (327, V17, "v17", V17, None),
    (328, V18, "v18", V18, None),
    (329, V19, "v19", V19, None),
    (330, V20, "v20", V20, None),
    (331, V21, "v21", V21, None),
    (332, V22, "v22", V22, None),
    (333, V23, "v23", V23, None),
    (334, V24, "v24", V24, None),
    (335, V25, "v25", V25, None),
    (336, V26, "v26", V26, None),
    (337, V27, "v27", V27, None),
    (338, V28, "v28", V28, None),
    (339, V29, "v29", V29, None),
    (340, V30, "v30", V30, None),
    (341, V31, "v31", V31, None)
);

enum_from_pairs!(
    X64Reg,
    (1, Ah, "ah", Rax, None),
    (2, Al, "al", Rax, None),
    (3, Ax, "ax", Rax, None),
    (4, Bh, "bh", Rbx, None),
    (5, Bl, "bl", Rbx, None),
    (6, Bp, "bp", Rbp, None),
    (7, Bpl, "bpl", Rbp, None),
    (8, Bx, "bx", Rbx, None),
    (9, Ch, "ch", Rcx, None),
    (10, Cl, "cl", Rcx, None),
    (11, Cs, "cs", Cs, None),
    (12, Cx, "cx", Rcx, None),
    (13, Dh, "dh", Rdx, None),
    (14, Di, "di", Rdi, None),
    (15, Dil, "dil", Rdi, None),
    (16, Dl, "dl", Rdx, None),
    (17, Ds, "ds", Ds, None),
    (18, Dx, "dx", Rdx, None),
    (19, Eax, "eax", Rax, None),
    (20, Ebp, "ebp", Rbp, None),
    (21, Ebx, "ebx", Rbx, None),
    (22, Ecx, "ecx", Rcx, None),
    (23, Edi, "edi", Rdi, None),
    (24, Edx, "edx", Rdx, None),
    (25, Rflags, "rflags", Rflags, None),
    (26, Eip, "eip", Rip, None),
    (27, Eiz, "eiz", Eiz, None), // Riz?
    (28, Es, "es", Es, None),
    (29, Esi, "esi", Rsi, None),
    (30, Esp, "esp", Rsp, None),
    (31, Fpsw, "fpsw", Fpsw, None),
    (32, Fs, "fs", Fs, None),
    (33, Gs, "gs", Gs, None),
    (34, Ip, "ip", Ip, None),
    (35, Rax, "rax", Rax, Some(0)),
    (36, Rbp, "rbp", Rbp, Some(5)),
    (37, Rbx, "rbx", Rbx, Some(3)),
    (38, Rcx, "rcx", Rcx, Some(1)),
    (39, Rdi, "rdi", Rdi, Some(7)),
    (40, Rdx, "rdx", Rdx, Some(2)),
    (41, Rip, "rip", Rip, None),
    (42, Riz, "riz", Riz, None),
    (43, Rsi, "rsi", Rsi, Some(6)),
    (44, Rsp, "rsp", Rsp, Some(4)),
    (45, Si, "si", Rsi, None),
    (46, Sil, "sil", Rsi, None),
    (47, Sp, "sp", Rsp, None),
    (48, Spl, "spl", Rsp, None),
    (49, Ss, "ss", Ss, None),
    (50, Cr0, "cr0", Cr0, None),
    (51, Cr1, "cr1", Cr1, None),
    (52, Cr2, "cr2", Cr2, None),
    (53, Cr3, "cr3", Cr3, None),
    (54, Cr4, "cr4", Cr4, None),
    (55, Cr5, "cr5", Cr5, None),
    (56, Cr6, "cr6", Cr6, None),
    (57, Cr7, "cr7", Cr7, None),
    (58, Cr8, "cr8", Cr8, None),
    (59, Cr9, "cr9", Cr9, None),
    (60, Cr10, "cr10", Cr10, None),
    (61, Cr11, "cr11", Cr11, None),
    (62, Cr12, "cr12", Cr12, None),
    (63, Cr13, "cr13", Cr13, None),
    (64, Cr14, "cr14", Cr14, None),
    (65, Cr15, "cr15", Cr15, None),
    (66, Dr0, "dr0", Dr0, None),
    (67, Dr1, "dr1", Dr1, None),
    (68, Dr2, "dr2", Dr2, None),
    (69, Dr3, "dr3", Dr3, None),
    (70, Dr4, "dr4", Dr4, None),
    (71, Dr5, "dr5", Dr5, None),
    (72, Dr6, "dr6", Dr6, None),
    (73, Dr7, "dr7", Dr7, None),
    (74, Dr8, "dr8", Dr8, None),
    (75, Dr9, "dr9", Dr9, None),
    (76, Dr10, "dr10", Dr10, None),
    (77, Dr11, "dr11", Dr11, None),
    (78, Dr12, "dr12", Dr12, None),
    (79, Dr13, "dr13", Dr13, None),
    (80, Dr14, "dr14", Dr14, None),
    (81, Dr15, "dr15", Dr15, None),
    (82, Fp0, "fp0", Fp0, None),
    (83, Fp1, "fp1", Fp1, None),
    (84, Fp2, "fp2", Fp2, None),
    (85, Fp3, "fp3", Fp3, None),
    (86, Fp4, "fp4", Fp4, None),
    (87, Fp5, "fp5", Fp5, None),
    (88, Fp6, "fp6", Fp6, None),
    (89, Fp7, "fp7", Fp7, None),
    (90, K0, "k0", K0, None),
    (91, K1, "k1", K1, None),
    (92, K2, "k2", K2, None),
    (93, K3, "k3", K3, None),
    (94, K4, "k4", K4, None),
    (95, K5, "k5", K5, None),
    (96, K6, "k6", K6, None),
    (97, K7, "k7", K7, None),
    (98, Mm0, "mm0", Mm0, None),
    (99, Mm1, "mm1", Mm1, None),
    (100, Mm2, "mm2", Mm2, None),
    (101, Mm3, "mm3", Mm3, None),
    (102, Mm4, "mm4", Mm4, None),
    (103, Mm5, "mm5", Mm5, None),
    (104, Mm6, "mm6", Mm6, None),
    (105, Mm7, "mm7", Mm7, None),
    (106, R8, "r8", R8, Some(8)),
    (107, R9, "r9", R9, Some(9)),
    (108, R10, "r10", R10, Some(10)),
    (109, R11, "r11", R11, Some(11)),
    (110, R12, "r12", R12, Some(12)),
    (111, R13, "r13", R13, Some(13)),
    (112, R14, "r14", R14, Some(14)),
    (113, R15, "r15", R15, Some(15)),
    (114, St0, "st(0)", St0, None),
    (115, St1, "st(1)", St1, None),
    (116, St2, "st(2)", St2, None),
    (117, St3, "st(3)", St3, None),
    (118, St4, "st(4)", St4, None),
    (119, St5, "st(5)", St5, None),
    (120, St6, "st(6)", St6, None),
    (121, St7, "st(7)", St7, None),
    (122, Xmm0, "xmm0", Xmm0, None),
    (123, Xmm1, "xmm1", Xmm1, None),
    (124, Xmm2, "xmm2", Xmm2, None),
    (125, Xmm3, "xmm3", Xmm3, None),
    (126, Xmm4, "xmm4", Xmm4, None),
    (127, Xmm5, "xmm5", Xmm5, None),
    (128, Xmm6, "xmm6", Xmm6, None),
    (129, Xmm7, "xmm7", Xmm7, None),
    (130, Xmm8, "xmm8", Xmm8, None),
    (131, Xmm9, "xmm9", Xmm9, None),
    (132, Xmm10, "xmm10", Xmm10, None),
    (133, Xmm11, "xmm11", Xmm11, None),
    (134, Xmm12, "xmm12", Xmm12, None),
    (135, Xmm13, "xmm13", Xmm13, None),
    (136, Xmm14, "xmm14", Xmm14, None),
    (137, Xmm15, "xmm15", Xmm15, None),
    (138, Xmm16, "xmm16", Xmm16, None),
    (139, Xmm17, "xmm17", Xmm17, None),
    (140, Xmm18, "xmm18", Xmm18, None),
    (141, Xmm19, "xmm19", Xmm19, None),
    (142, Xmm20, "xmm20", Xmm20, None),
    (143, Xmm21, "xmm21", Xmm21, None),
    (144, Xmm22, "xmm22", Xmm22, None),
    (145, Xmm23, "xmm23", Xmm23, None),
    (146, Xmm24, "xmm24", Xmm24, None),
    (147, Xmm25, "xmm25", Xmm25, None),
    (148, Xmm26, "xmm26", Xmm26, None),
    (149, Xmm27, "xmm27", Xmm27, None),
    (150, Xmm28, "xmm28", Xmm28, None),
    (151, Xmm29, "xmm29", Xmm29, None),
    (152, Xmm30, "xmm30", Xmm30, None),
    (153, Xmm31, "xmm31", Xmm31, None),
    (154, Ymm0, "ymm0", Ymm0, None),
    (155, Ymm1, "ymm1", Ymm1, None),
    (156, Ymm2, "ymm2", Ymm2, None),
    (157, Ymm3, "ymm3", Ymm3, None),
    (158, Ymm4, "ymm4", Ymm4, None),
    (159, Ymm5, "ymm5", Ymm5, None),
    (160, Ymm6, "ymm6", Ymm6, None),
    (161, Ymm7, "ymm7", Ymm7, None),
    (162, Ymm8, "ymm8", Ymm8, None),
    (163, Ymm9, "ymm9", Ymm9, None),
    (164, Ymm10, "ymm10", Ymm10, None),
    (165, Ymm11, "ymm11", Ymm11, None),
    (166, Ymm12, "ymm12", Ymm12, None),
    (167, Ymm13, "ymm13", Ymm13, None),
    (168, Ymm14, "ymm14", Ymm14, None),
    (169, Ymm15, "ymm15", Ymm15, None),
    (170, Ymm16, "ymm16", Ymm16, None),
    (171, Ymm17, "ymm17", Ymm17, None),
    (172, Ymm18, "ymm18", Ymm18, None),
    (173, Ymm19, "ymm19", Ymm19, None),
    (174, Ymm20, "ymm20", Ymm20, None),
    (175, Ymm21, "ymm21", Ymm21, None),
    (176, Ymm22, "ymm22", Ymm22, None),
    (177, Ymm23, "ymm23", Ymm23, None),
    (178, Ymm24, "ymm24", Ymm24, None),
    (179, Ymm25, "ymm25", Ymm25, None),
    (180, Ymm26, "ymm26", Ymm26, None),
    (181, Ymm27, "ymm27", Ymm27, None),
    (182, Ymm28, "ymm28", Ymm28, None),
    (183, Ymm29, "ymm29", Ymm29, None),
    (184, Ymm30, "ymm30", Ymm30, None),
    (185, Ymm31, "ymm31", Ymm31, None),
    (186, Zmm0, "zmm0", Zmm0, None),
    (187, Zmm1, "zmm1", Zmm1, None),
    (188, Zmm2, "zmm2", Zmm2, None),
    (189, Zmm3, "zmm3", Zmm3, None),
    (190, Zmm4, "zmm4", Zmm4, None),
    (191, Zmm5, "zmm5", Zmm5, None),
    (192, Zmm6, "zmm6", Zmm6, None),
    (193, Zmm7, "zmm7", Zmm7, None),
    (194, Zmm8, "zmm8", Zmm8, None),
    (195, Zmm9, "zmm9", Zmm9, None),
    (196, Zmm10, "zmm10", Zmm10, None),
    (197, Zmm11, "zmm11", Zmm11, None),
    (198, Zmm12, "zmm12", Zmm12, None),
    (199, Zmm13, "zmm13", Zmm13, None),
    (200, Zmm14, "zmm14", Zmm14, None),
    (201, Zmm15, "zmm15", Zmm15, None),
    (202, Zmm16, "zmm16", Zmm16, None),
    (203, Zmm17, "zmm17", Zmm17, None),
    (204, Zmm18, "zmm18", Zmm18, None),
    (205, Zmm19, "zmm19", Zmm19, None),
    (206, Zmm20, "zmm20", Zmm20, None),
    (207, Zmm21, "zmm21", Zmm21, None),
    (208, Zmm22, "zmm22", Zmm22, None),
    (209, Zmm23, "zmm23", Zmm23, None),
    (210, Zmm24, "zmm24", Zmm24, None),
    (211, Zmm25, "zmm25", Zmm25, None),
    (212, Zmm26, "zmm26", Zmm26, None),
    (213, Zmm27, "zmm27", Zmm27, None),
    (214, Zmm28, "zmm28", Zmm28, None),
    (215, Zmm29, "zmm29", Zmm29, None),
    (216, Zmm30, "zmm30", Zmm30, None),
    (217, Zmm31, "zmm31", Zmm31, None),
    (218, R8B, "r8b", R8, None),
    (219, R9B, "r9b", R9, None),
    (220, R10B, "r10b", R10, None),
    (221, R11B, "r11b", R11, None),
    (222, R12B, "r12b", R12, None),
    (223, R13B, "r13b", R13, None),
    (224, R14B, "r14b", R14, None),
    (225, R15B, "r15b", R15, None),
    (226, R8D, "r8d", R8, None),
    (227, R9D, "r9d", R9, None),
    (228, R10D, "r10d", R10, None),
    (229, R11D, "r11d", R11, None),
    (230, R12D, "r12d", R12, None),
    (231, R13D, "r13d", R13, None),
    (232, R14D, "r14d", R14, None),
    (233, R15D, "r15d", R15, None),
    (234, R8W, "r8w", R8, None),
    (235, R9W, "r9w", R9, None),
    (236, R10W, "r10w", R10, None),
    (237, R11W, "r11w", R11, None),
    (238, R12W, "r12w", R12, None),
    (239, R13W, "r13w", R13, None),
    (240, R14W, "r14w", R14, None),
    (241, R15W, "r15w", R15, None),
    (242, Bnd0, "bnd0", Bnd0, None),
    (243, Bnd1, "bnd1", Bnd1, None),
    (244, Bnd2, "bnd2", Bnd2, None),
    (245, Bnd3, "bnd3", Bnd3, None)
);

#[derive(Clone, Copy, Debug)]
pub enum Reg {
    Aarch64Reg(Aarch64Reg),
    X64Reg(X64Reg),
}

impl Reg {
    pub fn from_num(arch: Arch, num: u16) -> Option<Self> {
        Some(match arch {
            Arch::ARM64 => Reg::Aarch64Reg(Aarch64Reg::from_num(num)?),
            Arch::X86_64 => Reg::X64Reg(X64Reg::from_num(num)?),
        })
    }

    pub fn canonical(self) -> Self {
        match self {
            Reg::Aarch64Reg(r) => Reg::Aarch64Reg(r.canonical()),
            Reg::X64Reg(r) => Reg::X64Reg(r.canonical()),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Reg::Aarch64Reg(r) => r.as_str(),
            Reg::X64Reg(r) => r.as_str(),
        }
    }

    pub fn from_idx(arch: Arch, i: usize) -> Option<Self> {
        Some(match arch {
            Arch::ARM64 => Reg::Aarch64Reg(Aarch64Reg::from_idx(i)?),
            Arch::X86_64 => Reg::X64Reg(X64Reg::from_idx(i)?),
        })
    }

    pub fn idx(&self) -> Option<usize> {
        match self {
            Reg::Aarch64Reg(r) => r.idx(),
            Reg::X64Reg(r) => r.idx(),
        }
    }
}

#[cfg(test)]
mod tests {
    use capstone::RegId;
    #[allow(unused_imports)]
    use convert_case::Casing;

    use super::{Aarch64Reg, X64Reg};
    use crate::{
        arch::Arch,
        state::{Aarch64State, State, X64State},
    };

    #[test]
    fn aarch64_canon() {
        assert_eq!(Aarch64Reg::W0.canonical(), Aarch64Reg::X0);
        assert_eq!(Aarch64Reg::X5.canonical(), Aarch64Reg::X5);
        assert_eq!(Aarch64Reg::Sp.canonical(), Aarch64Reg::Sp);
        assert_eq!(Aarch64Reg::Xzr.canonical(), Aarch64Reg::Xzr);
        assert_eq!(Aarch64Reg::Lr.canonical(), Aarch64Reg::Lr);
        assert_eq!(Aarch64Reg::W30.canonical(), Aarch64Reg::Lr);
    }

    #[test]
    fn aarch64_reg_names() {
        let arch = Arch::ARM64;
        let cs = arch.make_capstone().unwrap();

        for i in 0..=u16::MAX {
            let i = RegId(i);

            let cs_name = cs.reg_name(i);
            let our_name = Aarch64Reg::from_num(i.0).map(|reg| reg.as_str().to_string());

            if cs_name != our_name {
                panic!("{cs_name:?} != {our_name:?}")
            }

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

    #[test]
    fn aarch64_idxs() {
        for i in 0..32 {
            let r = Aarch64Reg::from_idx(i);
            assert!(r.is_some(), "no reg with idx {}", i);

            let r = r.unwrap();
            assert_eq!(r.as_str(), Aarch64State::reg_name_idx(i));
        }

        assert_eq!(Aarch64Reg::from_idx(32), None);
    }

    #[test]
    fn x64_idxs() {
        for i in 0..16 {
            let r = X64Reg::from_idx(i);
            assert!(r.is_some(), "no reg with idx {}", i);

            let r = r.unwrap();
            assert_eq!(r.as_str(), X64State::reg_name_idx(i));
        }

        assert_eq!(X64Reg::from_idx(16), None);
    }

    #[test]
    fn x64_reg_names() {
        let arch = Arch::X86_64;
        let cs = arch.make_capstone().unwrap();

        for i in 0..=u16::MAX {
            let i = RegId(i);

            let cs_name = cs.reg_name(i);
            let our_name = X64Reg::from_num(i.0).map(|reg| reg.as_str().to_string());

            if cs_name != our_name {
                panic!("{cs_name:?} != {our_name:?}")
            }

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
