pub enum RegFunc {
    Src1,
    Src2,
    Dest,
}

pub struct Register<'a> {
    pub name: &'a str,
    pub number: u32,
}

pub const REG_FILE: [Register; 33] = [
    Register {
        name: "zero",
        number: 0,
    },
    Register {
        name: "ra",
        number: 1,
    },
    Register {
        name: "sp",
        number: 2,
    },
    Register {
        name: "gp",
        number: 3,
    },
    Register {
        name: "tp",
        number: 4,
    },
    Register {
        name: "t0",
        number: 5,
    },
    Register {
        name: "t1",
        number: 6,
    },
    Register {
        name: "t2",
        number: 7,
    },
    Register {
        name: "fp",
        number: 8,
    },
    Register {
        name: "s0",
        number: 8,
    },
    Register {
        name: "s1",
        number: 9,
    },
    Register {
        name: "a0",
        number: 10,
    },
    Register {
        name: "a1",
        number: 11,
    },
    Register {
        name: "a2",
        number: 12,
    },
    Register {
        name: "a3",
        number: 13,
    },
    Register {
        name: "a4",
        number: 14,
    },
    Register {
        name: "a5",
        number: 15,
    },
    Register {
        name: "a6",
        number: 16,
    },
    Register {
        name: "a7",
        number: 17,
    },
    Register {
        name: "s2",
        number: 18,
    },
    Register {
        name: "s3",
        number: 19,
    },
    Register {
        name: "s4",
        number: 20,
    },
    Register {
        name: "s5",
        number: 21,
    },
    Register {
        name: "s6",
        number: 22,
    },
    Register {
        name: "s7",
        number: 23,
    },
    Register {
        name: "s8",
        number: 24,
    },
    Register {
        name: "s9",
        number: 25,
    },
    Register {
        name: "s10",
        number: 26,
    },
    Register {
        name: "s11",
        number: 27,
    },
    Register {
        name: "t3",
        number: 28,
    },
    Register {
        name: "t4",
        number: 29,
    },
    Register {
        name: "t5",
        number: 30,
    },
    Register {
        name: "t6",
        number: 31,
    },
];
