const RAM_SIZE:usize = 65536;
struct CPU {
    pc:u16, // Program Counter
    sp:u16, // Stack Pointer
    ram:[u8; RAM_SIZE],
    //Registers
    a:u8, //Primary Accumulator
    b:u8,
    c:u8,
    d:u8,
    e:u8,
    h:u8,
    l:u8,
    // Flags
    s:bool, // Sign bit, set if result neg
    z:bool, // Zero bit, set if res zero
    p:bool, // Parity bit, set if number of 1 bits in res is even
    cy:bool, // Carry bit  
    ac:bool, // Aux carry
}

impl CPU {
    pub fn new() -> Self {
        let mut new_cpu = Self {
            pc:0,
            sp:0,
            ram:[0; RAM_SIZE],
            a:0,
            b:0,
            c:0,
            d:0,
            e:0,
            h:0,
            l:0,
            s:false,
            z:false,
            p:false,
            cy:false,
            ac:false,
        };


        new_cpu
    }

    pub fn reset(&mut self) {
        self.pc = 0;
        self.sp = 0;
        self.ram = [0; RAM_SIZE];
        self.a = 0;
        self.b = 0;
        self.c = 0;
        self.d = 0;
        self.e = 0;
        self.h = 0;
        self.l = 0;
        self.s = false;
        self.z = false;
        self.p = false;
        self.cy = false;
        self.ac = false;
    }

    pub fn tick(&mut self) {
        //Fetch
        let op:u8 = self.fetch();
        //Decode
        //Execute
    }

    fn fetch(&mut self) -> u8 {
        let opcode = self.ram[self.pc as usize] as u8;
        self.pc += 1;
        opcode
    }

    fn execute(&mut self, op:u8) {
        let digit_1 = (op & 0xF0) >> 4;
        let digit_2 = op & 0x0F;
        
        // Parity Bit optimizaion to implement (GPT3.5) - self.p = answer.count_ones() & 1 == 0;

        match (digit_1, digit_2) {
            // NOP
            (0, 0) => return,
            
            //LXI B, D16
            (0,1) => {
                /*
                3 Byte instruction, (OP/C-Byte/B-Byte)
                3 MCycles Op Fetch/Mem Read/Mem Read
                */
                let low_byte = self.ram[self.pc as usize] as u8;
                let high_byte = self.ram[(self.pc + 1) as usize];
                
                self.c = low_byte;
                self.b = high_byte;
                self.pc += 3;

            }
            
            //STAX B
            (0, 2) => {
                /*
                1 Byte
                Store A at memory location of (BC) 
                */
                let high_byte = self.b as u16;
                let low_byte = self.c as u16;
                let addr = (high_byte << 8) | low_byte;

                self.ram[addr as usize] = self.a;
            }

            //INX B
            (0,3) => {
                /*
                1 Byte
                Increments B and C by one, does not affect flags
                */
                self.b = self.b.wrapping_add(1);
                self.c = self.c.wrapping_add(1);

            }

            //INR B
            (0,4) => {
                /*
                1 Byte
                Increments B, flags = Z, S, P, AC
                */
                let answer:u8 = self.b.wrapping_add(1);
                self.z = (answer & 0xFF) == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;
                self.ac = answer & 0xF == 0;

                self.b = answer as u8;
            }

            //DCR B
            (0,5) => {
                /*
                1 Byte
                Decrements (BC), flags = Z, S, P, AC
                */

                let high_byte = self.b as u16;
                let low_byte = self.c as u16;
                let addr = (high_byte << 8) | low_byte;

                let answer = self.ram[addr as usize];

                let answer = answer.wrapping_sub(1);
                self.z = (answer & 0xFF) == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;
                self.ac = answer & 0xF == 0;
            }

            //MVI B, D8
            (0, 6) => {
                /*
                2 Byte
                Moves byte 2 to B                
                */
                let byte_2 = self.ram[(self.pc + 1) as usize];

                self.b = byte_2;

                self.pc += 2;

            }

            //RLC
            (0, 7) => {
                /*
                1 Byte
                Rotate Accumulator Left, sets CY to LMB shits A by 1 and concats CY to A
                */

                self.cy = (self.a & 0x80) != 0;
                self.a = self.a << 1;

                self.a |= self.cy as u8;


            }

            //*NOP (should not be used, alt opcode)
            (0, 8) => return,

            //DAD B
            (0, 9) => {
                /*
                1 Byte
                Double Add BC + HL -> HL CY flag
                */
                let bc_16:u16 = ((self.b as u16) << 8) | (self.c as u16);
                let hl_16:u16 = ((self.h as u16) << 8) | (self.l as u16);

                let (answer, carry) = bc_16.overflowing_add(hl_16);

                self.cy = carry;

                self.h = (answer >> 8) as u8;
                self.l = answer as u8;
            }

            //LDAX B
            (0, 0xA) => {
                /*
                1 Byte
                Loads A from memory location (BC)
                */
                let addr:u16 = ((self.b as u16) << 8) | (self.c as u16);
                self.a = self.ram[addr as usize]; 
            }

            //DCX B
            (0, 0xB) => {
                /*
                1 Byte
                Decrements Register Pair
                */
                self.b = self.b.wrapping_sub(1);
                self.c = self.c.wrapping_sub(1);
            }

            //INR C
            (0,0xC) => {
                /*
                1 Byte
                Increments C, flags = Z, S, P, AC
                */
                let answer:u8 = self.c.wrapping_add(1);
                self.z = (answer & 0xFF) == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;
                self.ac = answer & 0xF == 0;

                self.c = answer as u8;
            }

            //DCR C
            (0,0xD) => {
                /*
                1 Byte
                Increments CD, flags = Z, S, P, AC
                */

                let high_byte = self.c as u16;
                let low_byte = self.d as u16;
                let addr = (high_byte << 8) | low_byte;

                let answer = self.ram[addr as usize];

                let answer = answer.wrapping_sub(1);
                self.z = (answer & 0xFF) == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;
                self.ac = answer & 0xF == 0;
            }

            //MVI C, D8
            (0, 0xE) => {
                /*
                2 Byte
                Moves byte 2 to C                
                */
                let byte_2 = self.ram[(self.pc + 1) as usize];

                self.c = byte_2;

                self.pc += 2;

            }

            //RRC
            (0, 7) => {
                /*
                1 Byte
                Rotate Accumulator Right, sets CY to LMB shits A by 1 and concats CY to A
                */
                self.cy = (self.a & 0x01) != 0;

                self.a = self.a >> 1;

                self.a |= ((self.cy as u8) >> 7);

            }

            //*NOP (should not be used, alt opcode)
            (1, 0) => return,

            //LXI D, D16
            (1,1) => {
                /*
                3 Byte instruction, (OP/E-Byte/D-Byte)
                3 MCycles Op Fetch/Mem Read/Mem Read
                */
                let low_byte = self.ram[self.pc as usize] as u8;
                let high_byte = self.ram[(self.pc + 1) as usize];
                
                self.e = low_byte;
                self.d = high_byte;
                self.pc += 3;

            }

            //STAX D
            (1, 2) => {
                /*
                1 Byte
                Store A at memory location of (BC) 
                */
                let high_byte = self.d as u16;
                let low_byte = self.e as u16;
                let addr = (high_byte << 8) | low_byte;

                self.ram[addr as usize] = self.a;
            }

            //INX D
            (1,3) => {
                /*
                1 Byte
                Increments D and E by one, does not affect flags
                */
                self.d = self.d.wrapping_add(1);
                self.e = self.e.wrapping_add(1);

            }

            //INR D
            (1,4) => {
                /*
                1 Byte
                Increments D, flags = Z, S, P, AC
                */
                let answer:u16 = (self.d as u16) + 1;
                self.z = (answer & 0xFF) == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;
                self.ac = answer & 0xF == 0;

                self.d = answer as u8;
            }

            //DCR D
            (1,5) => {
                /*
                1 Byte
                Decrements (DE), flags = Z, S, P, AC
                */

                let high_byte = self.d as u16;
                let low_byte = self.e as u16;
                let addr = (high_byte << 8) | low_byte;

                let answer = self.ram[addr as usize];

                let answer = answer.wrapping_sub(1);
                self.z = (answer & 0xFF) == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;
                self.ac = answer & 0xF == 0;
            }

            //MVI D, D8
            (1, 6) => {
                /*
                2 Byte
                Moves byte 2 to D                
                */
                let byte_2 = self.ram[(self.pc + 1) as usize];

                self.d = byte_2;

                self.pc += 2;

            }

            //RAL
            (1, 7) => {
                /*
                1 Byte
                Rotates Accumulator left through carry
                */

                let carry_bit = self.cy;

                self.cy = self.a & 0x80 == 1;

                self.a = (self.a << 1) | (carry_bit as u8);

            }

            //DAD D
            (1, 9) => {
                /*
                1 Byte
                Double Add DE + HL -> HL CY flag
                */
                let bc_16:u16 = ((self.d as u16) << 8) | (self.e as u16);
                let hl_16:u16 = ((self.h as u16) << 8) | (self.l as u16);

                let (answer, carry) = bc_16.overflowing_add(hl_16);

                self.cy = carry;

                self.h = (answer >> 8) as u8;
                self.l = answer as u8;
            }

             //LDAX D
             (1, 0xA) => {
                /*
                1 Byte
                Loads A from memory location (DE)
                */
                let addr:u16 = ((self.d as u16) << 8) | (self.e as u16);
                self.a = self.ram[addr as usize]; 
            }

            //DCX D
            (1, 0xB) => {
                /*
                1 Byte
                Decrements Register Pair
                */
                self.d = self.d.wrapping_sub(1);
                self.e = self.e.wrapping_sub(1);
            }

            //INR E
            (1, 0xC) => {
                /*
                1 Byte
                Increments E, flags = Z, S, P, AC
                */
                let answer:u8 = self.e.wrapping_add(1);
                self.z = (answer & 0xFF) == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;
                self.ac = answer & 0xF == 0;

                self.e = answer as u8;
            }

            //DCR E
            (1,0xD) => {
                /*
                1 Byte
                Decrements (EH), flags = Z, S, P, AC
                */

                let high_byte = self.e as u16;
                let low_byte = self.h as u16;
                let addr = (high_byte << 8) | low_byte;

                let answer = self.ram[addr as usize];

                let answer = answer.wrapping_sub(1);
                self.z = (answer & 0xFF) == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;
                self.ac = answer & 0xF == 0;
            }

            //MVI E, D8
            (1, 0xE) => {
                /*
                2 Byte
                Moves byte 2 to E                
                */
                let byte_2 = self.ram[(self.pc + 1) as usize];

                self.e = byte_2;

                self.pc += 2;

            }

            //RAR
            (1, 0xF) => {
                /*
                1 Byte
                Rotates Accumulator right through carry
                */

                let carry_bit = self.cy;

                self.cy = self.a & 0x1 == 1;

                self.a = (self.a >> 1) | ((carry_bit as u8) << 7);

            }

            //NOP*
            (2, 0) => return,

            //LXI H, D16
            (2,1) => {
                /*
                3 Byte instruction, (OP/C-Byte/B-Byte)
                3 MCycles Op Fetch/Mem Read/Mem Read
                */
                let low_byte = self.ram[self.pc as usize] as u8;
                let high_byte = self.ram[(self.pc + 1) as usize];
                
                self.c = low_byte;
                self.b = high_byte;
                self.pc += 2;

            }

            (_, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }
}