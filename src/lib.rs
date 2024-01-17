use core::panic;
use std::fs::File;
use std::io::Write;

const RAM_SIZE:usize = 65536; //64 KiB
pub struct CPU {
    pub pc:u16, // Program Counter
    pub sp:u16, // Stack Pointer
    pub ram:[u8; RAM_SIZE],
    //Registers
    pub a:u8, //Primary Accumulator
    pub b:u8,
    pub c:u8,
    pub d:u8,
    pub e:u8,
    pub h:u8,
    pub l:u8,
    // Flags
    s:bool, // Sign bit, set if result neg
    z:bool, // Zero bit, set if res zero
    p:bool, // Parity bit, set if number of 1 bits in res is even
    cy:bool, // Carry bit  
    ac:bool, // Aux carry
    pub int_enabled:bool, // Interrupt bit
    pub last_interrupt:u8,
    pub cycles:u32,

    //IO API (TODO)
    //try_input:bool,
    //try_output:bool,
    //in_port:u8,
    pub out_port:u8,
}

impl CPU {
    pub fn new() -> Self {
        let new_cpu = Self {
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
            int_enabled:true,
            last_interrupt:16,
            cycles: 0,
            //try_input:false,
            //try_output:false,
            //in_port:0,
            out_port:255,
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
        self.int_enabled = false;
        self.last_interrupt = 16;
        self.cycles = 0;
        //self.try_input = false;
        //self.try_output = false;
        //self.in_port = 0;
        self.out_port = 255;
    }

    pub fn init_start_addr(&mut self, start_addr:u16) {
        self.pc = start_addr;
    }

    pub fn tick(&mut self) {
        //Fetch & Decode
        let op:u8 = self.fetch();
        //Execute
        self.execute(op);
    }

    pub fn debug_tick (&mut self) -> String {
        //Fetch & Decode
        let op:u8 = self.fetch();
        //Execute
        self.execute(op);

        let mut flag_value:u8 = 0;
        let flag_vec:Vec<bool> = vec![self.s, self.z, false, self.ac, false, self.p, false, self.cy];
        let mut flags = ["","","","",""];

        for (i, &flag) in flag_vec.iter().enumerate() {
            if flag {
                flag_value |= 1 << (7 - i);
            }
        }

        flags[0] = if flag_vec[0] {"S"} else {"."};
        flags[1] = if flag_vec[1] {"Z"} else {"."};
        flags[2] = if flag_vec[3] {"AC"} else {"."};
        flags[3] = if flag_vec[5] {"P"} else {"."};
        flags[4] = if flag_vec[7] {"CY"} else {"."};

        let af = (self.a as u16) << 8 | (flag_value as u16);
        let bc = (self.b as u16) << 8 | (self.c as u16);
        let de = (self.d as u16) << 8 | (self.e as u16);
        let hl = (self.h as u16) << 8 | (self.l as u16);

        


        let output = format!("AF-{:04x} BC-{:04x} DE-{:04x} HL-{:04x} PC-{:04x} SP-{:04x} (SP)-{:04x} OP-{:02x} Flags-{}{}{}{}{}", af, bc, de, hl, self.pc, self.sp, self.ram[self.sp as usize], op, flags[0], flags[1], flags[2], 
        flags[3], flags[4]);

        output
    }

    pub fn gui_debug_tick (&mut self) -> (Vec<u16>, Vec<&str>){
        //Fetch & Decode
        let op:u8 = self.fetch();
        //Execute
        self.execute(op);

        let mut flag_value:u8 = 0;
        let flag_vec:Vec<bool> = vec![self.s, self.z, false, self.ac, false, self.p, false, self.cy];
        let mut flags = vec!["","","","",""];

        for (i, &flag) in flag_vec.iter().enumerate() {
            if flag {
                flag_value |= 1 << (7 - i);
            }
        }
        flags[0] = if flag_vec[0] {"S"} else {"."};
        flags[1] = if flag_vec[1] {"Z"} else {"."};
        flags[2] = if flag_vec[3] {"AC"} else {"."};
        flags[3] = if flag_vec[5] {"P"} else {"."};
        flags[4] = if flag_vec[7] {"CY"} else {"."};

        let cpu_flags = vec![(self.a as u16) << 8 | (flag_value as u16), (self.b as u16) << 8 | (self.c as u16), 
                                        (self.d as u16) << 8 | (self.e as u16), (self.h as u16) << 8 | (self.l as u16), op as u16];
        
        (cpu_flags, flags)
    }

    pub fn load(&mut self, data: &[u8]) {
        let end = data.len();

        self.ram[0..end].copy_from_slice(data);
    }

    pub fn load_from(&mut self, data:&[u8], start:usize) {
        let end = data.len() + start;
        self.ram[start..end].copy_from_slice(data);
    }

    fn fetch(&mut self) -> u8 {
        let opcode = self.ram[self.pc as usize] as u8;
        self.pc += 1;
        opcode
    }

    fn execute(&mut self, op:u8) {
        let digit_1 = (op & 0xF0) >> 4;
        let digit_2 = op & 0x0F;

        match (digit_1, digit_2) {
            // NOP
            (0, 0) => {
                self.cycles += 4;
                return
            },
            
            //LXI B, D16
            (0, 1) => {
                /*
                3 Byte instruction, (OP/C-Byte/B-Byte)
                3 MCycles Op Fetch/Mem Read/Mem Read
                */
                let low_byte = self.ram[self.pc as usize] as u8;
                let high_byte = self.ram[(self.pc + 1) as usize];
                
                self.c = low_byte;
                self.b = high_byte;

                self.pc += 2;
                self.cycles += 10;
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

                self.cycles += 7;
            }

            //INX B
            (0, 3) => {
                /*
                1 Byte
                Increments BC by one, does not affect flags
                */
                let mut bc:u16 = (self.b as u16) << 8 | (self.c as u16);
                bc = bc.wrapping_add(1);
                
                self.b = (bc >> 8) as u8;
                self.c = bc as u8;

                self.cycles += 5;
            }

            //INR B
            (0, 4) => {
                /*
                1 Byte
                Increments B, flags = Z, S, P, AC
                */
                let answer = self.b.overflowing_add(1);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.b & 0x0F) < (answer.0 & 0x0F);

                self.b = answer.0;

                self.cycles += 5;
            }

            //DCR B
            (0, 5) => {
                /*
                1 Byte
                Decrements B, flags = Z, S, P, AC
                */

                let answer = self.b.overflowing_sub(1);

                self.z = answer.0 == 0;
                self.s = answer.0 & 0x80 != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.b & 0x0F) > (answer.0 & 0x0F);

                self.b = answer.0;

                self.cycles += 5;
            }

            //MVI B, D8
            (0, 6) => {
                /*
                2 Byte
                Moves byte 2 to B                
                */
                let byte_2 = self.ram[(self.pc) as usize];

                self.b = byte_2;

                self.pc += 1;
                self.cycles += 7;
            }

            //RLC
            (0, 7) => {
                /*
                1 Byte
                Rotate Accumulator Left, sets CY to LMB shits A by 1 and concats CY to A
                */

                self.cy = (self.a & 0x80) != 0;

                self.a = self.a.rotate_left(1);

                self.cycles += 4;
            }

            //*NOP (should not be used, alt opcode)
            (0, 8) => {
                self.cycles += 4;
                return
            },

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

                self.cycles += 10;
            }

            //LDAX B
            (0, 0xA) => {
                /*
                1 Byte
                Loads A from memory location (BC)
                */
                let addr:u16 = ((self.b as u16) << 8) | (self.c as u16);
                self.a = self.ram[addr as usize]; 

                self.cycles += 7;
            }

            //DCX B
            (0, 0xB) => {
                /*
                1 Byte
                Decrements Register Pair
                */

                let mut bc_16:u16 = ((self.b as u16) << 8) | (self.c as u16);

                bc_16 = bc_16.wrapping_sub(1);

                self.b = (bc_16 >> 8) as u8;
                self.c = bc_16 as u8;

                self.cycles += 5;
            }

            //INR C
            (0,0xC) => {
                /*
                1 Byte
                Increments C, flags = Z, S, P, AC
                */
                let answer = self.c.overflowing_add(1);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.c & 0x0F) < (answer.0 & 0x0F);

                self.c = answer.0;

                self.cycles += 5;
            }

            //DCR C
            (0,0xD) => {
                /*
                1 Byte
                Decrements C, flags = Z, S, P, AC
                */

                let answer = self.c.overflowing_sub(1);

                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.c & 0x0F) > (answer.0 & 0x0F);

                self.c = answer.0;

                self.cycles += 5;
            }

            //MVI C, D8
            (0, 0xE) => {
                /*
                2 Byte
                Moves byte 2 to C                
                */
                let byte_2 = self.ram[(self.pc) as usize];

                self.c = byte_2;

                self.pc += 1;
                self.cycles += 7;
            }

            //RRC
            (0, 0xF) => {
                /*
                1 Byte
                Rotate Accumulator Right, sets CY to LMB shits A by 1 and concats CY to A
                */

                self.cy = (self.a & 1) != 0;

                self.a = self.a.rotate_right(1);

                self.cycles += 4;
            }

            //*NOP (should not be used, alt opcode)
            (1, 0) => {
                self.cycles += 4;
                return
            },

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

                self.pc += 2;
                self.cycles += 10;
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

                self.cycles += 7;
            }

            //INX D
            (1,  3) => {
                /*
                1 Byte
                Increments D and E by one, does not affect flags
                */
                let mut de:u16 = (self.d as u16) << 8 | (self.e as u16);
                de = de.wrapping_add(1);
                
                self.d = (de >> 8) as u8;
                self.e = de as u8;

                self.cycles += 5;
            }

            //INR D
            (1,4) => {
                /*
                1 Byte
                Increments D, flags = Z, S, P, AC
                */
                let answer = self.d.overflowing_add(1);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.d & 0x0F) < (answer.0 & 0x0F);

                self.d = answer.0;

                self.cycles += 5;
            }

            //DCR D
            (1,5) => {
                /*
                1 Byte
                Decrements D, flags = Z, S, P, AC
                */

                let answer = self.d.overflowing_sub(1);

                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.d & 0x0F) > (answer.0 & 0x0F);

                self.d = answer.0;

                self.cycles += 5;
            }

            //MVI D, D8
            (1, 6) => {
                /*
                2 Byte
                Moves byte 2 to D                
                */
                let byte_2 = self.ram[(self.pc) as usize];

                self.d = byte_2;

                self.pc += 1;
                self.cycles += 7;
            }

            //RAL
            (1, 7) => {
                /*
                1 Byte
                Rotates Accumulator left through carry
                */

                let carry_bit = self.a & 0x80 != 0;

                self.a = (self.a << 1) | (self.cy as u8);

                self.cy = carry_bit;

                self.cycles += 4;
            }

            //*NOP
            (1, 8) => {
                self.cycles += 4;
                return
            },

            //DAD D
            (1, 9) => {
                /*
                1 Byte
                Double Add DE + HL -> HL CY flag
                */
                let de_16:u16 = ((self.d as u16) << 8) | (self.e as u16);
                let hl_16:u16 = ((self.h as u16) << 8) | (self.l as u16);

                let (answer, carry) = de_16.overflowing_add(hl_16);

                self.cy = carry;

                self.h = (answer >> 8) as u8;
                self.l = answer as u8;

                self.cycles += 10;
            }

            //LDAX D
            (1, 0xA) => {
                /*
                1 Byte
                Loads A from memory location (DE)
                */
                let addr:u16 = ((self.d as u16) << 8) | (self.e as u16);
                self.a = self.ram[addr as usize]; 

                self.cycles += 7;
            }

            //DCX D
            (1, 0xB) => {
                /*
                1 Byte
                Decrements Register Pair
                */
                let mut de_16:u16 = ((self.d as u16) << 8) | (self.e as u16);

                de_16 = de_16.wrapping_sub(1);

                self.d = (de_16 >> 8) as u8;
                self.e = de_16 as u8;

                self.cycles += 5;
            }

            //INR E
            (1, 0xC) => {
                /*
                1 Byte
                Increments E, flags = Z, S, P, AC
                */
                let answer = self.e.overflowing_add(1);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.e & 0x0F) < (answer.0 & 0x0F);

                self.e = answer.0;

                self.cycles += 5;
            }

            //DCR E
            (1,0xD) => {
                /*
                1 Byte
                Decrements E, flags = Z, S, P, AC
                */

                let answer = self.e.overflowing_sub(1);

                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.e & 0x0F) > (answer.0 & 0x0F);

                self.e = answer.0;

                self.cycles += 5;
            }

            //MVI E, D8
            (1, 0xE) => {
                /*
                2 Byte
                Moves byte 2 to E                
                */
                let byte_2 = self.ram[(self.pc) as usize];

                self.e = byte_2;

                self.pc += 1;
                self.cycles += 7;
            }

            //RAR
            (1, 0xF) => {
                /*
                1 Byte
                Rotates Accumulator right through carry
                */

                let carry_bit = self.a & 1 != 0;

                self.a = (self.a >> 1) | ((self.cy as u8) << 7);

                self.cy = carry_bit;

                self.cycles += 4;
            }

            //*NOP
            (2, 0) => {
                self.cycles += 4;
                return
            },

            //LXI H, D16
            (2,1) => {
                /*
                3 Byte instruction, (OP/L-Byte/H-Byte)
                3 MCycles Op Fetch/Mem Read/Mem Read
                */
                let low_byte = self.ram[self.pc as usize] as u8;
                let high_byte = self.ram[(self.pc + 1) as usize];
                
                self.l = low_byte;
                self.h = high_byte;

                self.pc += 2;
                self.cycles += 10;
            }

            //SHLD adr
            (2, 2) => {
                /*
                3 Byte
                Stores L at (Address) and H at (Address) + 1
                */

                let low_byte = self.ram[self.pc as usize];
                let high_byte = self.ram[(self.pc + 1)  as usize];

                let addr = (high_byte as u16) << 8 | low_byte as u16;

                self.ram[addr as usize] = self.l;
                self.ram[(addr + 1) as usize] = self.h;
                
                self.pc += 2;
                self.cycles += 16;
                
                //TODO Implement Overflow Check
                
            }

            //INX H
            (2,3) => {
                /*
                1 Byte
                Increments H and L by one, does not affect flags
                */
                let mut hl:u16 = (self.h as u16) << 8 | (self.l as u16);
                hl = hl.wrapping_add(1);
                
                self.h = (hl >> 8) as u8;
                self.l = hl as u8;

                self.cycles += 5;
            }

            //INR H
            (2, 4) => {
                /*
                1 Byte
                Increments H, flags = Z, S, P, AC
                */
                let answer = self.h.overflowing_add(1);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.h & 0x0F) < (answer.0 & 0x0F);

                self.h = answer.0;

                self.cycles += 5;
            }

            //DCR H
            (2,5) => {
                /*
                1 Byte
                Decrements (HL), flags = Z, S, P, AC
                */

                let answer = self.h.overflowing_sub(1);

                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.h & 0x0F) > (answer.0 & 0x0F);

                self.h = answer.0;

                self.cycles += 5;
            }

            //MVI H, D8
            (2, 6) => {
                /*
                2 Byte
                Moves (byte 2) to H                
                */
                let byte_2 = self.ram[(self.pc) as usize];

                self.h = byte_2;

                self.pc += 1;
                self.cycles += 7;
            }

            //DAA
            (2, 7) => {
                /*
                1 Byte
                8 Bit accumulator is adjusted to form two 4 bit binary-coded-decimal digits
                Method as for i8080 manual
                */
                

                let mut ls_nibble = self.a & 0x0F;

                if self.ac || ls_nibble >= 10  { //ls_nibble is checked to see if greater than 9 or CY flag is set
                    self.a += 6; // 6 is added to ls_nibble

                    self.ac = (self.a & 0xF) < ls_nibble ; // AC flag is set if carry is present
                }

                ls_nibble = self.a & 0x0F;
                let mut ms_nibble = (self.a >> 4) & 0xF;

                if self.cy || ms_nibble > 9 { // ms_nibble is checked to see if greater than 9 or AC flag is set
                    ms_nibble += 6; // 6 is added to ms_nibble
                    self.cy = (self.a & 0xF) < ms_nibble; // AC flag is set if carry is present
                }

                self.a = (ms_nibble << 4) | ls_nibble; // Nibbles are merged after computing

                // Other Flags are set
                self.z = self.a == 0;
                self.s = (self.a & 0x80) != 0;
                self.p = (self.a.count_ones() % 2) == 0;

                self.cycles += 4;
            }

            //*NOP
            (2, 8) => {
                self.cycles += 4;
                return
            },

            //DAD H
            (2, 9) => {
                /*
                1 Byte
                Double Add HL + HL -> HL CY flag
                */
                let hl_16:u16 = ((self.h as u16) << 8) | (self.l as u16);

                let (answer, carry) = hl_16.overflowing_add(hl_16);

                self.cy = carry;

                self.h = (answer >> 8) as u8;
                self.l = answer as u8;

                self.cycles += 10;
            }

            //LHLD adr
            (2, 0xA) => {
                /*
                3 Byte
                Loads (Address) at L and (Adresss + 1) at H
                */

                let low_byte = self.ram[self.pc as usize];
                let high_byte = self.ram[(self.pc + 1)  as usize];

                let addr = ((high_byte as u16) << 8) | low_byte as u16;


                self.l = self.ram[addr as usize];
                self.h = self.ram[(addr + 1) as usize];

                self.pc += 2;
                self.cycles += 16;
            }

            //DCX H
            (2, 0xB) => {
                /*
                1 Byte
                Decrements Register Pair
                */
                let mut hl_16:u16 = ((self.h as u16) << 8) | (self.l as u16);

                hl_16 = hl_16.wrapping_sub(1);

                self.h = (hl_16 >> 8) as u8;
                self.l = hl_16 as u8;

                self.cycles += 5;
            }

            //INR L
            (2, 0xC) => {
                /*
                1 Byte
                Increments H, flags = Z, S, P, AC
                */
                let answer = self.l.overflowing_add(1);

                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.l & 0x0F) < (answer.0 & 0x0F);

                self.l = answer.0;

                self.cycles += 5;
            }

            //DCR L
            (2, 0xD) => {
                /*
                1 Byte
                Decrements H, flags = Z, S, P, AC
                */
                let answer = self.l.overflowing_sub(1);

                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.l & 0x0F) > (answer.0 & 0x0F);

                self.l = answer.0;

                self.cycles += 5;
            }

            //MVI L, D8
            (2, 0xE) => {
                /*
                2 Byte
                Moves (byte 2) to L                
                */
                let byte_2 = self.ram[(self.pc) as usize];

                self.l = byte_2;

                self.pc += 1;
                self.cycles += 7;
            }

            //CMA
            (2, 0xF) => {
                /*
                1 Byte
                Returns Ones compliment of accumulator
                */
                self.a = !self.a;

                self.cycles += 4;
            }

            //*NOP
            (3, 0) => {
                self.cycles += 4;
                return
            },

            //LXI , D16
            (3, 1) => {
                /*
                3 Byte instruction, (OP/L-Byte/H-Byte)
                3 MCycles Op Fetch/Mem Read/Mem Read
                */
                let low_byte = self.ram[self.pc as usize] as u16;
                let high_byte = self.ram[(self.pc + 1) as usize] as u16;

                self.sp = (high_byte << 8) | low_byte;

                self.pc += 2;
                self.cycles += 10;
            }

            //STA , D16
            (3, 2) => {
                /*
                3 Byte instruction, (OP/L-Byte/H-Byte)
                3 MCycles Op Fetch/Mem Read/Mem Read
                Stores A in two byte addr
                */
                let low_byte = self.ram[self.pc as usize];
                let high_byte = self.ram[(self.pc + 1) as usize];
                let addr = (high_byte as u16) << 8 | low_byte as  u16;

                self.ram[addr as usize] = self.a;
                
                self.pc += 2;
                self.cycles += 13;
            }

            //INX SP
            (3, 3) => {
                /*
                1 Byte
                SP = SP + 1
                */
                self.sp += 1;

                self.cycles += 5;
            }

            //INR M
            (3, 4) => {
                /*
                1 Byte
                Increments (HL), flags = Z, S, P, AC
                */

                let addr = ((self.h as u16) << 8) | self.l as u16;

                let answer = self.ram[addr as usize].overflowing_add(1);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.ram[addr as usize] & 0x0F) < (answer.0 & 0x0F);

                self.ram[addr as usize] = answer.0;

                self.cycles += 10;
            }

            //DCR M
            (3, 5) => {
                /*
                1 Byte
                Decrements (HL), flags = Z, S, P, AC
                */

                let addr = ((self.h as u16) << 8) | self.l as u16;

                let answer = self.ram[addr as usize].overflowing_sub(1);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.ram[addr as usize] & 0x0F) > (answer.0 & 0x0F);

                self.ram[addr as usize] = answer.0;

                self.cycles += 10;
            }

            //MVI M, D8
            (3, 6) => {
                /*
                2 Byte
                Moves byte 2 to (HL)                
                */

                let byte_2 = self.ram[(self.pc) as usize];
                let addr = ((self.h as u16) << 8) | (self.l as u16);

                self.ram[addr as usize] = byte_2;

                self.pc += 1;
                self.cycles += 10;
            }

            //STC
            (3, 7) => {
                /*
                1 Byte
                Sets CY flag
                */
                
                self.cy = true;

                self.cycles += 4;
            }

            //*NOP
            (3, 8) => {
                self.cycles += 4;
                return
            },

            //DAD SP
            (3, 9) => {
                /*
                1 Byte
                Double Add SP + HL -> HL CY flag
                */
                let hl_16:u16 = ((self.h as u16) << 8) | (self.l as u16);

                let (answer, carry) = hl_16.overflowing_add(self.sp);

                self.cy = carry;

                self.h = (answer >> 8) as u8;
                self.l = answer as u8;

                self.cycles += 10;
            }

            //LDA addr
            (3, 0xA) => {
                /*
                3 Byte
                Loads A from addr
                */
                
                let low_byte:u8 = self.ram[self.pc as usize];
                let high_byte:u8 = self.ram[(self.pc + 1) as usize]; 

                let addr:u16 = ((high_byte as u16) << 8) | (low_byte as u16);
                self.a = self.ram[addr as usize]; 

                self.pc += 2;
            }

            //DCX SP
            (3, 0xB) => {
                /*
                1 Byte
                Decrements SP
                */
                
                self.sp = self.sp.wrapping_sub(1);

                self.cycles += 5;
            }

            //INR A
            (3, 0xC) => {
                /*
                1 Byte
                Increments A, flags = Z, S, P, AC
                */
                let answer = self.a.overflowing_add(1);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) < (answer.0 & 0x0F);

                self.a = answer.0;

                self.cycles += 5;
            }

            //DCR A
            (3, 0xD) => {
                /*
                1 Byte
                Decrements A, flags = Z, S, P, AC
                */
                let answer = self.a.overflowing_sub(1);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) > (answer.0 & 0x0F);

                self.a = answer.0;

                self.cycles += 5;
            }

            //MVI A, D8
            (3, 0xE) => {
                /*
                2 Byte
                Moves (byte 2) to A               
                */
                let byte_2 = self.ram[(self.pc) as usize];

                self.a = byte_2;

                self.pc += 1;
                self.cycles += 7;
            }

            //CMC
            (3, 0xF) => {
                /*
                1 Byte
                Inverts CY
                */

                self.cy = !self.cy;

                self.cycles += 4;
            }

            //MOV B, B
            (4, 0) => {
                /*
                1 Byte
                Moves data from B to B
                */

                self.b = self.b;

                self.cycles += 5;
            }
            
            //MOV B, C
            (4, 1) => {
                /*
                1 Byte
                Moves data from C to B
                */

                self.b = self.c;

                self.cycles += 5;
            }

            //MOV B, D
            (4, 2) => {
                /*
                1 Byte
                Moves data from D to B
                */

                self.b = self.d;

                self.cycles += 5;
            }

            //MOV B, E
            (4, 3) => {
                /*
                1 Byte
                Moves data from E to B
                */

                self.b = self.e;

                self.cycles += 5;
            }

            //MOV B, H
            (4, 4) => {
                /*
                1 Byte
                Moves data from H to B
                */

                self.b = self.h;

                self.cycles += 5;
            }

            //MOV B, L
            (4, 5) => {
                /*
                1 Byte
                Moves data from L to B
                */

                self.b = self.l;

                self.cycles += 5;
            }

            //MOV B, M
            (4, 6) => {
                /*
                1 Byte
                Moves data from M (HL) to B
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.b = self.ram[addr as usize];

                self.cycles += 7;
            }

            //MOV B, A
            (4, 7) => {
                /*
                1 Byte
                Moves data from A to B
                */

                self.b = self.a;

                self.cycles += 5;
            }

            //MOV C, B
            (4, 8) => {
                /*
                1 Byte
                Moves data from B to C
                */

                self.c = self.b;

                self.cycles += 5;
            }

            //MOV C, C
            (4, 9) => {
                /*
                1 Byte
                Moves data from C to C
                */

                self.c = self.c;

                self.cycles += 5;
            }

            //MOV C, D
            (4, 0xA) => {
                /*
                1 Byte
                Moves data from D to C
                */

                self.c = self.d;

                self.cycles += 5;
            }

            //MOV C, E
            (4, 0xB) => {
                /*
                1 Byte
                Moves data from E to C
                */

                self.c = self.e;

                self.cycles += 5;
            }

            //MOV C, H
            (4, 0xC) => {
                /*
                1 Byte
                Moves data from H to C
                */

                self.c = self.h;

                self.cycles += 5;
            }

            //MOV C, L
            (4, 0xD) => {
                /*
                1 Byte
                Moves data from L to C
                */

                self.c = self.l;

                self.cycles += 5;
            }

            //MOV C, M
            (4, 0xE) => {
                /*
                1 Byte
                Moves data from M (HL) to C
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.c = self.ram[addr as usize];

                self.cycles += 7;
            }

            //MOV C, A
            (4, 0xF) => {
                /*
                1 Byte
                Moves data from A to C
                */

                self.c = self.a;

                self.cycles += 5;
            }
            
            //MOV D, B
            (5, 0) => {
                /*
                1 Byte
                Moves data from B to D
                */

                self.d = self.b;

                self.cycles += 5;
            }

            //MOV D, C
            (5, 1) => {
                /*
                1 Byte
                Moves data from C to D
                */

                self.d = self.c;

                self.cycles += 5;
            }

            //MOV D, D
            (5, 2) => {
                /*
                1 Byte
                Moves data from D to D
                */

                self.d = self.d;

                self.cycles += 5;
            }

            //MOV D, E
            (5, 3) => {
                /*
                1 Byte
                Moves data from E to D
                */

                self.d = self.e;

                self.cycles += 5;
            }

            //MOV D, H
            (5, 4) => {
                /*
                1 Byte
                Moves data from H to D
                */

                self.d = self.h;

                self.cycles += 5;
            }

            //MOV D, L
            (5, 5) => {
                /*
                1 Byte
                Moves data from L to D
                */

                self.d = self.l;

                self.cycles += 5;
            }

            //MOV D, M
            (5, 6) => {
                /*
                1 Byte
                Moves data from M (HL) to D
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.d = self.ram[addr as usize];

                self.cycles += 7;
            }

            //MOV D, A
            (5, 7) => {
                /*
                1 Byte
                Moves data from A to D
                */

                self.d = self.a;

                self.cycles += 5;
            }

            //MOV E, B
            (5, 8) => {
                /*
                1 Byte
                Moves data from B to E
                */

                self.e = self.b;

                self.cycles += 5;
            }

            //MOV E, C
            (5, 9) => {
                /*
                1 Byte
                Moves data from C to E
                */

                self.e = self.c;

                self.cycles += 5;
            }

            //MOV E, D
            (5, 0xA) => {
                /*
                1 Byte
                Moves data from D to E
                */

                self.e = self.d;

                self.cycles += 5;
            }

            //MOV E, E
            (5, 0xB) => {
                /*
                1 Byte
                Moves data from E to E
                */

                self.e = self.e;

                self.cycles += 5;
            }

            //MOV E, H
            (5, 0xC) => {
                /*
                1 Byte
                Moves data from H to E
                */

                self.e = self.h;

                self.cycles += 5;
            }

            //MOV E, L
            (5, 0xD) => {
                /*
                1 Byte
                Moves data from L to E
                */

                self.e = self.l;

                self.cycles += 5;
            }


            //MOV E, M
            (5, 0xE) => {
                /*
                1 Byte
                Moves data from M (HL) to E
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.e = self.ram[addr as usize];

                self.cycles += 7;
            }

            //MOV E, A
            (5, 0xF) => {
                /*
                1 Byte
                Moves data from A to E
                */

                self.e = self.a;

                self.cycles += 5;
            }

            //MOV H, B
            (6, 0) => {
                /*
                1 Byte
                Moves data from B to H
                */

                self.h = self.b;

                self.cycles += 5;
            }

            //MOV H, C
            (6, 1) => {
                /*
                1 Byte
                Moves data from C to H
                */

                self.h = self.c;

                self.cycles += 5;
            }

            //MOV H, D
            (6, 2) => {
                /*
                1 Byte
                Moves data from D to H
                */

                self.h = self.d;

                self.cycles += 5;
            }

            //MOV H, E
            (6, 3) => {
                /*
                1 Byte
                Moves data from E to H
                */

                self.h = self.e;

                self.cycles += 5;
            }

            //MOV H, H
            (6, 4) => {
                /*
                1 Byte
                Moves data from H to H
                */

                self.h = self.h;

                self.cycles += 5;
            }

            //MOV H, L
            (6, 5) => {
                /*
                1 Byte
                Moves data from L to H
                */

                self.h = self.l;

                self.cycles += 5;
            }

            //MOV H, M
            (6, 6) => {
                /*
                1 Byte
                Moves data from M (HL) to H
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.h = self.ram[addr as usize];

                self.cycles += 7;
            }

            //MOV H, A
            (6, 7) => {
                /*
                1 Byte
                Moves data from A to H
                */

                self.h = self.a;

                self.cycles += 5;
            }

            //MOV L, B
            (6, 8) => {
                /*
                1 Byte
                Moves data from B to L
                */

                self.l = self.b;

                self.cycles += 5;
            }

            //MOV L, C
            (6, 9) => {
                /*
                1 Byte
                Moves data from C to L
                */

                self.l = self.c;
                
                self.cycles += 5;
            }

            //MOV L, D
            (6, 0xA) => {
                /*
                1 Byte
                Moves data from D to L
                */

                self.l = self.d;

                self.cycles += 5;
            }

            //MOV L, E
            (6, 0xB) => {
                /*
                1 Byte
                Moves data from E to L
                */

                self.l = self.e;

                self.cycles += 5;
            }

            //MOV L, H
            (6, 0xC) => {
                /*
                1 Byte
                Moves data from H to L
                */

                self.l = self.h;

                self.cycles += 5;
            }

            //MOV L, L
            (6, 0xD) => {
                /*
                1 Byte
                Moves data from L to L
                */

                self.l = self.l;

                self.cycles += 5;
            }

            //MOV L, M
            (6, 0xE) => {
                /*
                1 Byte
                Moves data from M (HL) to L
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.l = self.ram[addr as usize];

                self.cycles += 7;
            }

            //MOV L, A
            (6, 0xF) => {
                /*
                1 Byte
                Moves data from A to L
                */

                self.l = self.a;

                self.cycles += 5;
            }

            //MOV M, B
            (7, 0) => {
                /*
                1 Byte
                Moves data from B to M (HL)
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.ram[addr as usize] = self.b;

                self.cycles += 7;
            }

            //MOV M, C
            (7, 1) => {
                /*
                1 Byte
                Moves data from C to M (HL)
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.ram[addr as usize] = self.c;

                self.cycles += 7;
            }

            //MOV M, D
            (7, 2) => {
                /*
                1 Byte
                Moves data from D to M (HL)
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.ram[addr as usize] = self.d;

                self.cycles += 7;
            }
            
            //MOV M, E
            (7, 3) => {
                /*
                1 Byte
                Moves data from E to M (HL)
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.ram[addr as usize] = self.e;

                self.cycles += 7;
            }

            //MOV M, H
            (7, 4) => {
                /*
                1 Byte
                Moves data from H to M (HL)
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.ram[addr as usize] = self.h;

                self.cycles += 7;
            }

            //MOV M, L
            (7, 5) => {
                /*
                1 Byte
                Moves data from L to M (HL)
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.ram[addr as usize] = self.l;

                self.cycles += 7;
            }

            //HLT
            (7, 6) => {
                //TODO
                
                self.cycles += 7;
                unimplemented!("Called 0x76 HLT")
            }

            //MOV M, A
            (7, 7) => {
                /*
                1 Byte
                Moves data from A to M (HL)
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.ram[addr as usize] = self.a;

                self.cycles += 7;
            }

            //MOV A, B
            (7, 8) => {
                /*
                1 Byte
                Moves B to A
                */

                self.a = self.b;

                self.cycles += 5;
            }

            //MOV A, B
            (7, 9) => {
                /*
                1 Byte
                Moves C to A
                */

                self.a = self.c;

                self.cycles += 5;
            }

            //MOV A, D
            (7, 0xA) => {
                /*
                1 Byte
                Moves data from D to A
                */

                self.a = self.d;

                self.cycles += 5;
            }

            //MOV A, E
            (7, 0xB) => {
                /*
                1 Byte
                Moves data from E to A
                */

                self.a = self.e;

                self.cycles += 5;
            }

            //MOV A, H
            (7, 0xC) => {
                /*
                1 Byte
                Moves data from H to A
                */

                self.a = self.h;

                self.cycles += 5;
            }

            //MOV A, L
            (7, 0xD) => {
                /*
                1 Byte
                Moves data from L to A
                */

                self.a = self.l;

                self.cycles += 5;
            }

            //MOV A, M
            (7, 0xE) => {
                /*
                1 Byte
                Moves data from M to A
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.a = self.ram[addr as usize];

                self.cycles += 5;
            }

            //MOV A, A
            (7, 0xF) => {
                /*
                1 Byte
                Moves data from A to A
                */

                self.a = self.a;

                self.cycles += 5;
            }

            //ADD B
            (8, 0) => {
                /*
                1 Byte
                Adds B to A, Flags - Z, S, P, CY, AC 
                */

                let answer = self.a.overflowing_add(self.b);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) > (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 4;
            }

            //ADD C
            (8, 1) => {
                /*
                1 Byte
                Adds C to A, Flags - Z, S, P, CY, AC 
                */

                let answer = self.a.overflowing_add(self.c);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) > (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 4;
            }

            //ADD D
            (8, 2) => {
                /*
                1 Byte
                Adds D to A, Flags - Z, S, P, CY, AC 
                */

                let answer = self.a.overflowing_add(self.d);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) > (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 4;
            }

            //ADD E
            (8, 3) => {
                /*
                1 Byte
                Adds C to A, Flags - Z, S, P, CY, AC 
                */

                let answer = self.a.overflowing_add(self.e);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) > (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 4;
            }

            //ADD H
            (8, 4) => {
                /*
                1 Byte
                Adds H to A, Flags - Z, S, P, CY, AC 
                */

                let answer = self.a.overflowing_add(self.h);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) > (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 4;
            }

            //ADD L
            (8, 5) => {
                /*
                1 Byte
                Adds L to A, Flags - Z, S, P, CY, AC 
                */

                let answer = self.a.overflowing_add(self.l);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) > (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 4;
            }

            //ADD M
            (8, 6) => {
                /*
                1 Byte
                Adds M (HL) to A, Flags - Z, S, P, CY, AC 
                */
                
                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                let answer = self.a.overflowing_add(self.ram[addr as usize]);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) > (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 7;
            }

            //ADD A
            (8, 7) => {
                /*
                1 Byte
                Adds L to A, Flags - Z, S, P, CY, AC 
                */

                let answer = self.a.overflowing_add(self.a);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) > (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 4;
            }

            //ADC B
            (8, 8) => {
                /*
                1 Byte
                Adds B + A + CY, Flags - Z, S, P, CY, AC 
                */

                let (answer, carry) = self.a.overflowing_add(self.b);
                let (carry_answer, carry_2) = answer.overflowing_add(self.cy as u8);

                let sum_low_nibble = (self.a & 0x0F) + (self.b & 0x0F) + (self.cy as u8);

                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.ac = sum_low_nibble > 0x0F;
                self.cy = carry || carry_2;

                self.a = carry_answer;

                self.cycles += 4;
            }

            //ADC C
            (8, 9) => {
                /*
                1 Byte
                Adds C + A + CY, Flags - Z, S, P, CY, AC 
                */

                let (answer, carry) = self.a.overflowing_add(self.c);
                let (carry_answer, carry_2) = answer.overflowing_add(self.cy as u8);

                let sum_low_nibble = (self.a & 0x0F) + (self.c & 0x0F) + (self.cy as u8);
                
                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.ac = sum_low_nibble > 0x0F;
                self.cy = carry || carry_2;

                self.a = carry_answer;

                self.cycles += 4;
            }

            //ADC D
            (8, 0xA) => {
                /*
                1 Byte
                Adds D + A + CY, Flags - Z, S, P, CY, AC 
                */

                let (answer, carry) = self.a.overflowing_add(self.d);
                let (carry_answer, carry_2) = answer.overflowing_add(self.cy as u8);

                let sum_low_nibble = (self.a & 0x0F) + (self.d & 0x0F) + (self.cy as u8);

                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.ac = sum_low_nibble > 0x0F;
                self.cy = carry || carry_2;

                self.a = carry_answer;

                self.cycles += 4;
            }

            //ADC E
            (8, 0xB) => {
                /*
                1 Byte
                Adds E + A + CY, Flags - Z, S, P, CY, AC 
                */

                let (answer, carry) = self.a.overflowing_add(self.e);
                let (carry_answer, carry_2) = answer.overflowing_add(self.cy as u8);

                let sum_low_nibble = (self.a & 0x0F) + (self.e & 0x0F) + (self.cy as u8);

                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.ac = sum_low_nibble > 0x0F;
                self.cy = carry || carry_2;

                self.a = carry_answer;

                self.cycles += 4;
            }

            //ADC H
            (8, 0xC) => {
                /*
                1 Byte
                Adds H + A + CY, Flags - Z, S, P, CY, AC 
                */

                let (answer, carry) = self.a.overflowing_add(self.h);
                let (carry_answer, carry_2) = answer.overflowing_add(self.cy as u8);

                let sum_low_nibble = (self.a & 0x0F) + (self.h & 0x0F) + (self.cy as u8);

                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.ac = sum_low_nibble > 0x0F;
                self.cy = carry || carry_2;

                self.a = carry_answer;

                self.cycles += 4;
            }  

            //ADC L
            (8, 0xD) => {
                /*
                1 Byte
                Adds E + A + CY, Flags - Z, S, P, CY, AC 
                */

                let (answer, carry) = self.a.overflowing_add(self.l);
                let (carry_answer, carry_2) = answer.overflowing_add(self.cy as u8);

                let sum_low_nibble = (self.a & 0x0F) + (self.l & 0x0F) + (self.cy as u8);

                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.ac = sum_low_nibble > 0x0F;
                self.cy = carry || carry_2;

                self.a = carry_answer;

                self.cycles += 4;
            }

            //ADC M
            (8, 0xE) => {
                /*
                1 Byte
                Adds (HL) + A + CY, Flags - Z, S, P, CY, AC 
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);
                
                let (answer, carry) = self.a.overflowing_add(self.ram[addr as usize]);
                let (carry_answer, carry_2) = answer.overflowing_add(self.cy as u8);

                let sum_low_nibble = (self.a & 0x0F) + (self.ram[addr as usize] & 0x0F) + (self.cy as u8);

                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.ac = sum_low_nibble > 0x0F;
                self.cy = carry || carry_2;

                self.a = carry_answer;

                self.cycles += 7;
            }

            //ADC A
            (8, 0xF) => {
                /*
                1 Byte
                Adds A + A + CY, Flags - Z, S, P, CY, AC 
                */

                let (answer, carry) = self.a.overflowing_add(self.a);
                let (carry_answer, carry_2) = answer.overflowing_add(self.cy as u8);

                let sum_low_nibble = (self.a & 0x0F) + (self.a & 0x0F) + (self.cy as u8);

                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.ac = sum_low_nibble > 0x0F;
                self.cy = carry || carry_2;

                self.a = carry_answer;

                self.cycles += 4;
            }

            //SUB B
            (9, 0) => {
                /*
                1 Byte
                Subtract B and A, Flags - Z, S, P, CY, AC 
                */

                let answer = self.a.overflowing_sub(self.b);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) < (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 4;
            }

            //SUB C
            (9, 1) => {
                /*
                1 Byte
                Subtract C and A, Flags - Z, S, P, CY, AC 
                */

                let answer = self.a.overflowing_sub(self.c);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) < (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 4;
            }

            //SUB D
            (9, 2) => {
                /*
                1 Byte
                Subtract D and A, Flags - Z, S, P, CY, AC 
                */

                let answer = self.a.overflowing_sub(self.d);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) < (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 4;
            }

            //SUB E
            (9, 3) => {
                /*
                1 Byte
                Subtract C and A, Flags - Z, S, P, CY, AC 
                */

                let answer = self.a.overflowing_sub(self.e);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) < (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 4;
            }

            //SUB H
            (9, 4) => {
                /*
                1 Byte
                Subtract H and A, Flags - Z, S, P, CY, AC 
                */

                let answer = self.a.overflowing_sub(self.h);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) < (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 4;
            }

            //SUB L
            (9, 5) => {
                /*
                1 Byte
                Subtract L and A, Flags - Z, S, P, CY, AC 
                */

                let answer = self.a.overflowing_sub(self.l);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) < (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 4;
            }

            //SUB M
            (9, 6) => {
                /*
                1 Byte
                Subtract M (HL) and A, Flags - Z, S, P, CY, AC 
                */
                
                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                let answer = self.a.overflowing_sub(self.ram[addr as usize]);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) < (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 7;
            }

            //SUB A
            (9, 7) => {
                /*
                1 Byte
                Subtract L and A, Flags - Z, S, P, CY, AC 
                */

                let answer = self.a.overflowing_sub(self.a);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = (self.a & 0x0F) < (answer.0 & 0x0F);
                self.cy = answer.1;

                self.a = answer.0;

                self.cycles += 4;
            }

            //SBB B
            (9, 8) => {
                /*
                1 Byte
                Subtracts B - A - CY, Flags - Z, S, P, CY, AC 
                */

                let (answer, carry) = self.a.overflowing_sub(self.b);
                let (carry_answer, carry_2) = answer.overflowing_sub(self.cy as u8);

                let diff_low_nibble = (self.a & 0x0F).wrapping_sub(self.b & 0x0F).wrapping_sub(self.cy as u8);

                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.cy = carry || carry_2;
                self.ac = diff_low_nibble > (self.a & 0x0F) || (self.cy && (self.a & 0x0F) == 0);

                self.a = carry_answer;

                self.cycles += 4;
            }

            //SBB C
            (9, 9) => {
                /*
                1 Byte
                Subtracts C - A - CY, Flags - Z, S, P, CY, AC 
                */

                let (answer, carry) = self.a.overflowing_sub(self.c);
                let (carry_answer, carry_2) = answer.overflowing_sub(self.cy as u8);

                let diff_low_nibble = (self.a & 0x0F).wrapping_sub(self.c & 0x0F).wrapping_sub(self.cy as u8);

                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.cy = carry || carry_2;
                self.ac = diff_low_nibble > (self.a & 0x0F) || (self.cy && (self.a & 0x0F) == 0);

                self.a = carry_answer;

                self.cycles += 4;
            }

            //SBB D
            (9, 0xA) => {
                /*
                1 Byte
                Subtracts D - A - CY, Flags - Z, S, P, CY, AC 
                */

                let (answer, carry) = self.a.overflowing_sub(self.d);
                let (carry_answer, carry_2) = answer.overflowing_sub(self.cy as u8);

                let diff_low_nibble = (self.a & 0x0F).wrapping_sub(self.d & 0x0F).wrapping_sub(self.cy as u8);

                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.cy = carry || carry_2;
                self.ac = diff_low_nibble > (self.a & 0x0F) || (self.cy && (self.a & 0x0F) == 0);

                self.a = carry_answer;

                self.cycles += 4;
            }

            //SBB E
            (9, 0xB) => {
                /*
                1 Byte
                Subtracts E - A - CY, Flags - Z, S, P, CY, AC 
                */

                let (answer, carry) = self.a.overflowing_sub(self.e);
                let (carry_answer, carry_2) = answer.overflowing_sub(self.cy as u8);

                let diff_low_nibble = (self.a & 0x0F).wrapping_sub(self.e & 0x0F).wrapping_sub(self.cy as u8);

                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.cy = carry || carry_2;
                self.ac = diff_low_nibble > (self.a & 0x0F) || (self.cy && (self.a & 0x0F) == 0);

                self.a = carry_answer;

                self.cycles += 4;
            }

            //SBB H
            (9, 0xC) => {
                /*
                1 Byte
                Subtracts H - A - CY, Flags - Z, S, P, CY, AC 
                */

                let (answer, carry) = self.a.overflowing_sub(self.h);
                let (carry_answer, carry_2) = answer.overflowing_sub(self.cy as u8);

                let diff_low_nibble = (self.a & 0x0F).wrapping_sub(self.h & 0x0F).wrapping_sub(self.cy as u8);

                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.cy = carry || carry_2;
                self.ac = diff_low_nibble > (self.a & 0x0F) || (self.cy && (self.a & 0x0F) == 0);

                self.a = carry_answer;

                self.cycles += 4;
            }

            //SBB L
            (9, 0xD) => {
                /*
                1 Byte
                Subtracts E - A - CY, Flags - Z, S, P, CY, AC 
                */

                let (answer, carry) = self.a.overflowing_sub(self.l);
                let (carry_answer, carry_2) = answer.overflowing_sub(self.cy as u8);

                let diff_low_nibble = (self.a & 0x0F).wrapping_sub(self.l & 0x0F).wrapping_sub(self.cy as u8);

                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.cy = carry || carry_2;
                self.ac = diff_low_nibble > (self.a & 0x0F) || (self.cy && (self.a & 0x0F) == 0);

                self.a = carry_answer;

                self.cycles += 4;
            }

            //SBB M
            (9, 0xE) => {
                /*
                1 Byte
                Subtracts (HL) - A - CY, Flags - Z, S, P, CY, AC 
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);
                
                let (answer, carry) = self.a.overflowing_sub(self.ram[addr as usize]);
                let (carry_answer, carry_2) = answer.overflowing_sub(self.cy as u8);

                let diff_low_nibble = (self.a & 0x0F).wrapping_sub(self.ram[addr as usize] & 0x0F).wrapping_sub(self.cy as u8);

                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.cy = carry || carry_2;
                self.ac = diff_low_nibble > (self.a & 0x0F) || (self.cy && (self.a & 0x0F) == 0);

                self.a = carry_answer;

                self.cycles += 7;
            }

            //SBB A
            (9, 0xF) => {
                /*
                1 Byte
                Subtracts E - A - CY, Flags - Z, S, P, CY, AC 
                */

                let (answer, carry) = self.a.overflowing_sub(self.a);
                let (carry_answer, carry_2) = answer.overflowing_sub(self.cy as u8);

                let diff_low_nibble = (self.a & 0x0F).wrapping_sub(self.a & 0x0F).wrapping_sub(self.cy as u8);

                self.z = carry_answer == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = carry_answer.count_ones() % 2 == 0;
                self.cy = carry || carry_2;
                self.ac = diff_low_nibble > (self.a & 0x0F) || (self.cy && (self.a & 0x0F) == 0);

                self.a = carry_answer;

                self.cycles += 4;
            }

            //ANA B
            (0xA, 0) => {
                /*
                1 Byte
                A & B affects CY, Z, S, P, AC
                */

                let answer = self.a & self.b;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //ANA C
            (0xA, 1) => {
                /*
                1 Byte
                A & C affects CY, Z, S, P, AC
                */

                let answer = self.a & self.c;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //ANA D
            (0xA, 2) => {
                /*
                1 Byte
                A & D affects CY, Z, S, P, AC
                */

                let answer = self.a & self.d;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //ANA E
            (0xA, 3) => {
                /*
                1 Byte
                A & E affects CY, Z, S, P, AC
                */

                let answer = self.a & self.e;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //ANA H
            (0xA, 4) => {
                /*
                1 Byte
                A & H affects CY, Z, S, P, AC
                */

                let answer = self.a & self.h;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //ANA L
            (0xA, 5) => {
                /*
                1 Byte
                A & L affects CY, Z, S, P, AC
                */

                let answer = self.a & self.l;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //ANA M
            (0xA, 6) => {
                /*
                1 Byte
                A & (HL) affects CY, Z, S, P, AC
                */

                let addr:u16 = ((self.h as u16) << 8) | self.l as u16;

                let answer = self.a & self.ram[addr as usize];

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 7;
            }

            //ANA A
            (0xA, 7) => {
                /*
                1 Byte
                A & A affects CY, Z, S, P, AC
                */

                let answer = self.a & self.a;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //XRA B
            (0xA, 8) => {
                /*
                1 Byte
                A ^ B affects CY, Z, S, P, AC
                */

                let answer = self.a ^ self.b;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //XRA C
            (0xA, 9) => {
                /*
                1 Byte
                A ^ C affects CY, Z, S, P, AC
                */

                let answer = self.a ^ self.c;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //XRA D
            (0xA, 0xA) => {
                /*
                1 Byte
                A ^ D affects CY, Z, S, P, AC
                */

                let answer = self.a ^ self.d;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //XRA E
            (0xA, 0xB) => {
                /*
                1 Byte
                A ^ E affects CY, Z, S, P, AC
                */

                let answer = self.a ^ self.e;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //XRA H
            (0xA, 0xC) => {
                /*
                1 Byte
                A ^ H affects CY, Z, S, P, AC
                */

                let answer = self.a ^ self.h;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //XRA L
            (0xA, 0xD) => {
                /*
                1 Byte
                A ^ L affects CY, Z, S, P, AC
                */

                let answer = self.a ^ self.l;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //XRA M
            (0xA, 0xE) => {
                /*
                1 Byte
                A ^ (HL) affects CY, Z, S, P, AC
                */

                let addr:u16 = ((self.h as u16) << 8) | self.l as u16;

                let answer = self.a ^ self.ram[addr as usize];

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 7;
            }

            //XRA A
            (0xA, 0xF) => {
                /*
                1 Byte
                A ^ A affects CY, Z, S, P, AC
                */

                let answer = self.a ^ self.a;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

             //ORA B
            (0xB, 0) => {
                /*
                1 Byte
                A | B affects CY, Z, S, P, AC
                */

                let answer = self.a | self.b;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //ORA C
            (0xB, 1) => {
                /*
                1 Byte
                A | C affects CY, Z, S, P, AC
                */

                let answer = self.a | self.c;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //ORA D
            (0xB, 2) => {
                /*
                1 Byte
                A | D affects CY, Z, S, P, AC
                */

                let answer = self.a | self.d;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //ORA E
            (0xB, 3) => {
                /*
                1 Byte
                A | E affects CY, Z, S, P, AC
                */

                let answer = self.a | self.e;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //ORA H
            (0xB, 4) => {
                /*
                1 Byte
                A | H affects CY, Z, S, P, AC
                */

                let answer = self.a | self.h;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //ORA L
            (0xB, 5) => {
                /*
                1 Byte
                A | L affects CY, Z, S, P, AC
                */

                let answer = self.a | self.l;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //ORA M
            (0xB, 6) => {
                /*
                1 Byte
                A | (HL) affects CY, Z, S, P, AC
                */

                let addr:u16 = ((self.h as u16) << 8) | self.l as u16;

                let answer = self.a | self.ram[addr as usize];

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 7;
            }

            //ORA A
            (0xB, 7) => {
                /*
                1 Byte
                A | A affects CY, Z, S, P, AC
                */

                let answer = self.a | self.a;

                self.cy = false; //Resets carry bit
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.cycles += 4;
            }

            //CMP B
            (0xB, 8) => {
                /*
                1 Bytes
                Sets Flags based on comparison of A and B
                */

                let (answer, carry) = self.a.overflowing_sub(self.b);

                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = answer.count_ones() % 2 == 0;
                self.cy = carry;
                self.ac = (self.a & 0x0F) < (self.b & 0x0F);

                self.cycles += 4;
            }

            //CMP C
            (0xB, 9) => {
                /*
                1 Bytes
                Sets Flags based on comparison of A and C
                */

                let (answer, carry) = self.a.overflowing_sub(self.c);

                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = answer.count_ones() % 2 == 0;
                self.cy = carry;
                self.ac = (self.a & 0x0F) < (self.c & 0x0F);

                self.cycles += 4;
            }

            //CMP D
            (0xB, 0xA) => {
                /*
                1 Bytes
                Sets Flags based on comparison of A and D
                */

                let (answer, carry) = self.a.overflowing_sub(self.d);

                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = answer.count_ones() % 2 == 0;
                self.cy = carry;
                self.ac = (self.a & 0x0F) < (self.d & 0x0F);

                self.cycles += 4;
            }

            //CMP E
            (0xB, 0xB) => {
                /*
                1 Bytes
                Sets Flags based on comparison of A and E
                */

                let (answer, carry) = self.a.overflowing_sub(self.e);

                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = answer.count_ones() % 2 == 0;
                self.cy = carry;
                self.ac = (self.a & 0x0F) < (self.e & 0x0F);

                self.cycles += 4;
            }

            //CMP H
            (0xB, 0xC) => {
                /*
                1 Bytes
                Sets Flags based on comparison of A and H
                */

                let (answer, carry) = self.a.overflowing_sub(self.h);

                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = answer.count_ones() % 2 == 0;
                self.cy = carry;
                self.ac = (self.a & 0x0F) < (self.h & 0x0F);
                
                self.cycles += 4;
            }

            //CMP L
            (0xB, 0xD) => {
                /*
                1 Bytes
                Sets Flags based on comparison of A and L
                */

                let (answer, carry) = self.a.overflowing_sub(self.l);

                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = answer.count_ones() % 2 == 0;
                self.cy = carry;
                self.ac = (self.a & 0x0F) < (self.l & 0x0F);

                self.cycles += 4;
            }

            //CMP M
            (0xB, 0xE) => {
                /*
                1 Bytes
                Sets Flags based on comparison of A and (HL)
                */

                let addr = ((self.h as u16) << 8) | self.l as u16;

                let (answer, carry) = self.a.overflowing_sub(self.ram[addr as usize]);

                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = answer.count_ones() % 2 == 0;
                self.cy = carry;
                self.ac = (self.a & 0x0F) < (self.ram[addr as usize] & 0x0F);

                self.cycles += 7;
            }

            //CMP A
            (0xB, 0xF) => {
                /*
                1 Bytes
                Sets Flags based on comparison of A and A
                */

                let (answer, carry) = self.a.overflowing_sub(self.a);

                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = answer.count_ones() % 2 == 0;
                self.cy = carry;
                self.ac = (self.a & 0x0F) < (self.a & 0x0F);

                self.cycles += 4;
            }
            
            //RNZ
            (0xC, 0) => {
                /*
                1 Byte
                If Z not set, RET
                */

                if self.z == false {
                    let low_byte = self.ram[self.sp as usize] as u16;
                    let high_byte = self.ram[(self.sp + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.sp += 2;
                    self.cycles += 11;
                }
                else {
                    self.cycles += 5;
                    return;
                }
            }

            //POP B
            (0xC, 1) => {
                /*
                1 Byte
                Stores (SP) at C, (SP + 1) at B, SP + 2
                */

                self.c = self.ram[self.sp as usize];
                self.b = self.ram[(self.sp + 1) as usize];

                self.sp += 2;
                self.cycles += 10;
            }

            //JNZ adr
            (0xC, 2) => {
                /*
                3 Byte
                If Z not set then PC = addr
                */

                if self.z == false {
                    let addr = ((self.ram[(self.pc + 1) as usize] as u16) << 8) | ((self.ram[self.pc as usize]) as u16);
                    self.pc = addr;
                    self.cycles += 10;
                } else {
                    self.pc += 2;
                    self.cycles += 10;
                    return;
                }
            }

            //JMP addr
            (0xC, 3) => {
                /*
                3 Byte
                PC = Addr
                */
                let addr = ((self.ram[(self.pc + 1) as usize] as u16) << 8) | ((self.ram[self.pc as usize]) as u16);
                self.pc = addr;
                self.cycles += 10;
            }

            //CNZ addr
            (0xC, 4) => {
                /*
                3 Bytes
                If Z not set, CALL addr
                */

                if self.z == false {
                    self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                    self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                    
                    self.sp -= 2;

                    let low_byte = self.ram[self.pc as usize] as u16;
                    let high_byte = self.ram[(self.pc + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.cycles += 17;
                }
                else {
                    self.pc += 2;
                    self.cycles += 11;               
                }
                
            }

            //PUSH B
            (0xC, 5) => {
                /*
                1 Byte
                Stores C at (SP - 2), B at (SP - 1), SP - 2
                */

                self.ram[(self.sp - 2) as usize] = self.c;
                self.ram[(self.sp - 1) as usize] = self.b;

                self.sp -= 2;
                self.cycles += 11;
            }

            //ADI D8
            (0xC, 6) => {
                /*
                2 Byte
                Adds immediate to accumulator
                */
                let answer =self.a.overflowing_add(self.ram[self.pc as usize]);
                

                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = answer.1 == true;
                self.cy = answer.1 == true;

                self.a = answer.0;
                self.pc += 1;
                self.cycles += 7;
            }

            //RST 0
            (0xC, 7) => {
                /*
                1 Byte
                CALL $0
                */

                self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                
                self.sp -= 2;

                self.pc = 0x0;
                self.cycles += 11;
            }

            //RZ
            (0xC, 8) => {
                /*
                1 Byte
                If Z is set, RET
                */

                if self.z {
                        let low_byte = self.ram[self.sp as usize] as u16;
                    let high_byte = self.ram[(self.sp + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.sp += 2;
                    self.cycles += 11;
                }
                else {
                    self.cycles += 5;
                    return;
                }
            }

            //RET
            (0xC, 9) => {
                /*
                1 Byte
                PC.Low = (SP), PC.high = (SP + 1), SP + 2
                Subroutine Return
                */

                let low_byte = self.ram[self.sp as usize] as u16;
                let high_byte = self.ram[(self.sp + 1) as usize] as u16;

                self.pc = (high_byte << 8) | low_byte;
                self.sp += 2;
                self.cycles += 10;
            }

            //JZ addr
            (0xC, 0xA) => {
                /*
                3 Byte
                If Z is set, PC = Addr
                */
                if self.z {
                    let addr = ((self.ram[(self.pc + 1) as usize] as u16) << 8) | ((self.ram[self.pc as usize]) as u16);
                    self.pc = addr;
                    self.cycles += 10;
                }
                else {
                    self.pc += 2;
                    self.cycles += 10;
                    return;
                }
            }

            //*JMP addr
            (0xC, 0xB) => {
                /*
                3 Byte
                If Z is set, PC = Addr
                */

                let addr = ((self.ram[(self.pc + 1) as usize] as u16) << 8) | ((self.ram[self.pc as usize]) as u16);
                self.pc = addr;
                self.cycles += 10;
            }

            //CZ adr
            (0xC, 0xC) => {
                /*
                3 Byte
                If Z is set, CALL addr
                Call = (SP-1) = PC.high, (SP-2)= PC.lo, SP = SP-2, PC=addr
                Subroutine Call
                */

                if self.z {
                    self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                    self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                    
                    self.sp -= 2;

                    let low_byte = self.ram[self.pc as usize] as u16;
                    let high_byte = self.ram[(self.pc + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.cycles += 17;
                }
                else {
                    self.pc += 2;
                    self.cycles += 11;
                }
            }

            //CALL adr
            (0xC, 0xD) => {
                /*
                3 Byte
                (SP-1) = PC.high, (SP-2)= PC.lo, SP = SP-2, PC=addr
                Subroutine Call
                */

                let low_byte = self.ram[self.pc as usize] as u16;
                let high_byte = self.ram[(self.pc + 1) as usize] as u16;

                let address = (high_byte << 8) | low_byte;

                #[cfg(feature = "cpudiag")]
                if address == 5 {
                    if self.c == 9 {
                        let mut offset = ((self.d as u16) << 8) | self.e as u16;
                        loop {
                            let character = self.ram[offset as usize];

                            if character as char == '$' {
                                break;
                            } else {
                                offset += 1;
                            }
                            print!("{}", character as char);
                        }
                        print!("\n");
                    } else if address == 0 {
                        ::std::process::exit(0)
                    }
                    if self.c == 2 {
                        println!("{}", self.e as char)
                    }
                }
                
                /* if address == 5 {
                    if self.c == 9 {
                        let offset = ((self.d as u16) << 8) | self.e as u16;
                        let mut str:u8 = 0;
                        let mut count = 0;
                        while str != ('$' as u8) {
                            str = self.ram[(offset + 3 + count) as usize];
                            print!("{}", str as char);
                            count += 1;
                        }
                        print!("\n");
                        ::std::process::exit(0)

                    } else if address == 0 {
                        ::std::process::exit(0)
                    }
                } */

                self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                
                self.sp -= 2;

                self.pc = address;
                self.cycles += 17;
            }

            //ACI D8
            (0xC, 0xE) => {
                /*
                2 Byte
                Add A + Immediate Data + CY, affects Z, S, P, CY, AC
                */

                let (answer, carry) = self.a.overflowing_add(self.ram[self.pc as usize]);
                let (carry_answer, carry_2) = answer.overflowing_add(self.cy as u8);

                
                self.z = (carry_answer & 0xFF) == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = (carry_answer.count_ones() % 2) == 0;
                self.cy = carry || carry_2;
                self.ac = carry || carry_2;

                self.a = carry_answer;

                self.pc += 1;
                self.cycles += 7;
            }

            //RST 1
            (0xC, 0xF) => {
                /*
                1 Byte
                CALL $8 (0x0008)
                */

                self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                
                self.sp -= 2;

                self.pc = 0x8;
                self.cycles += 11;
            }

            //RNC
            (0xD, 0) => {
                /*
                1 Byte
                If CY not set, RET
                */

                if self.cy == false {
                    let low_byte = self.ram[self.sp as usize] as u16;
                    let high_byte = self.ram[(self.sp + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.sp += 2;
                    self.cycles += 11;
                }
                else {
                    self.cycles += 5;
                    return;
                }
            }

            //POP D
            (0xD, 1) => {
                /*
                1 Byte
                Stores (SP) at E, (SP + 1) at D, SP + 2
                */

                self.e = self.ram[self.sp as usize];
                self.d = self.ram[(self.sp + 1) as usize];

                self.sp += 2;
                self.cycles += 10;
            }

            //JNC Addr
            (0xD, 2) => {
                /*
                3 Byte
                If CY not set then PC = addr
                */

                if self.cy == false {
                    let addr = ((self.ram[(self.pc + 1) as usize] as u16) << 8) | ((self.ram[self.pc as usize]) as u16);
                    self.pc = addr;
                    self.cycles += 10;
                } else {
                    self.pc += 2;
                    self.cycles += 10;
                    return;

                }
            }

            //OUT D8
            (0xD, 3) => {

                //TODO IMPLEMENT OUT API

                self.out_port = self.ram[self.pc as usize];

                #[cfg(feature = "cputest")]
                match self.out_port { //Emulates CP/M Sys Calls
                    0 => {
                        ::std::process::exit(0)
                    }
                    1 => {
                        match self.c {
                            2 => {
                                println!("{}", self.e as char);
                                ::std::process::exit(0)
                            }
                            9 => {
                                let offset = ((self.d as u16) << 8) | self.e as u16;
                                let mut character:u8 = 0;
                                let mut count = 0;
                                while character != ('$' as u8) {
                                    character = self.ram[(offset + count) as usize];
                                    print!("{}", character as char);
                                    count += 1;
                                }
                                print!("\n");
                                ::std::process::exit(0)
                            }
                            _ => return

                        }
                    }

                    _ => return
                }

                self.pc += 1;
                self.cycles += 10;
            }

            //CNC addr
            (0xD, 4) => {
                /*
                3 Bytes
                If CY not set, CALL addr
                */

                if self.cy == false {
                    self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                    self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                    
                    self.sp -= 2;

                    let low_byte = self.ram[self.pc as usize] as u16;
                    let high_byte = self.ram[(self.pc + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.cycles += 17;
                }
                else {
                    self.pc += 2;
                    self.cycles += 11;
                }
                
            }

            //PUSH D
            (0xD, 5) => {
                /*
                1 Byte
                Stores E at (SP - 2), D at (SP - 1), SP - 2
                */

                self.ram[(self.sp - 2) as usize] = self.e;
                self.ram[(self.sp - 1) as usize] = self.d;

                self.sp -= 2;
                self.cycles += 11;
            }

            //SUI D8
            (0xD, 6) => {
                /*
                2 Byte
                Subtract A and Immediate Data, Flags - Z, S, P, CY, AC 
                */

                let data = self.ram[self.pc as usize];

                let answer = self.a.overflowing_sub(data);
                self.z = answer.0 == 0;
                self.s = (answer.0 & 0x80) != 0;
                self.p = answer.0.count_ones() % 2 == 0;
                self.ac = answer.1;
                self.cy = answer.1;

                self.a = answer.0;

                self.pc += 1;
                self.cycles += 7;
            }

            //RST 2
            (0xD, 7) => {
                /*
                1 Byte
                CALL $0 (0x0000)
                */

                self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                
                self.sp -= 2;

                self.pc = 0x10;
                self.cycles += 11;
            }

            //RC
            (0xD, 8) => {
                /*
                1 Byte
                If CY is set, RET
                */

                if self.cy {
                    let low_byte = self.ram[self.sp as usize] as u16;
                    let high_byte = self.ram[(self.sp + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.sp += 2;
                    self.cycles += 11;
                }
                else {
                    self.cycles += 5;
                    return;
                }
            }

            //*RET
            (0xD, 9) => {
                /*
                1 Byte
                PC.Low = (SP), PC.high = (SP + 1), SP + 2
                Subroutine Return
                */

                let low_byte = self.ram[self.sp as usize] as u16;
                let high_byte = self.ram[(self.sp + 1) as usize] as u16;

                self.pc = (high_byte << 8) | low_byte;
                self.sp += 2;
                self.cycles += 10;
            }
            
            //JC addr
            (0xD, 0xA) => {
                /*
                3 Byte
                If CY is set, PC = Addr
                */
                if self.cy {
                    let addr = ((self.ram[(self.pc + 1) as usize] as u16) << 8) | ((self.ram[self.pc as usize]) as u16);
                    self.pc = addr;
                    self.cycles += 10;
                }
                else {
                    self.pc += 2;
                    self.cycles += 10;
                    return;
                }
            }

            //IN D8
            (0xD, 0xB) => {
                /*
                2 Byte
                Write A with input from bort 2nd Byte
                */
                
                //TODO: Implement API
                
                self.pc += 1;
                self.cycles += 10;
            }

            //CC addr
            (0xD, 0xC) => {
                /*
                3 Bytes
                If CY is set, CALL addr
                */

                if self.cy{
                    self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                    self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                    
                    self.sp -= 2;

                    let low_byte = self.ram[self.pc as usize] as u16;
                    let high_byte = self.ram[(self.pc + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.cycles += 17;
                }
                else {
                    self.pc += 2;
                    self.cycles += 11;
                }
                
            }

            //*CALL
            (0xD, 0xD) => {
                /*
                3 Byte
                (SP-1) = PC.high, (SP-2)= PC.lo, SP = SP-2, PC=addr
                Subroutine Call
                */

                self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                
                self.sp -= 2;

                let low_byte = self.ram[self.pc as usize] as u16;
                let high_byte = self.ram[(self.pc + 1) as usize] as u16;

                self.pc = (high_byte << 8) | low_byte;
                self.cycles += 17;
            }

            //SBI D8
            (0xD, 0xE) => {
                /*
                2 Byte
                Subtract A - Immediate Data - CY, affects Z, S, P, CY, AC
                */

                let (answer, carry) = self.a.overflowing_sub(self.ram[self.pc as usize]);
                let (carry_answer, carry_2) = answer.overflowing_sub(self.cy as u8);

                
                self.z = (carry_answer & 0xFF) == 0;
                self.s = (carry_answer & 0x80) != 0;
                self.p = (carry_answer.count_ones() % 2) == 0;
                self.cy = carry || carry_2;
                self.ac = carry || carry_2;

                self.a = carry_answer;

                self.pc += 1;
                self.cycles += 7;
            }

            //RST 3
            (0xD, 0xF) => {
                /*
                1 Byte
                CALL $13
                */

                self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                
                self.sp -= 2;

                self.pc = 0x13;
                self.cycles += 11;
            }

            //RPO
            (0xE, 0) => {
                /*
                1 Byte
                If P not set (odd parity), RET
                */

                if self.p == false {
                    let low_byte = self.ram[self.sp as usize] as u16;
                    let high_byte = self.ram[(self.sp + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.sp += 2;
                    self.cycles += 11;
                }
                else {
                    self.cycles += 5;
                    return;
                }
            }


            //POP H
            (0xE, 1) => {
                /*
                1 Byte
                Stores (SP) at L, (SP + 1) at H, SP + 2
                */

                self.l = self.ram[self.sp as usize];
                self.h = self.ram[(self.sp + 1) as usize];
                
                self.sp += 2;

                self.cycles += 10;
            }

            //JPO addr
            (0xE, 2) => {
                /*
                3 Byte
                If P not set then PC = addr
                */

                if self.p == false {
                    let addr = ((self.ram[(self.pc + 1) as usize] as u16) << 8) | ((self.ram[self.pc as usize]) as u16);
                    self.pc = addr;
                    self.cycles += 10;
                } else {
                    self.pc += 2;
                    self.cycles += 10;
                    return;

                }
            }

            //XTHL
            (0xE, 3) => {
                /*
                1 Byte
                Exchange values in L/(SP) and H(SP + 1)
                */

                let mut xchng_byte = self.h;

                self.h = self.ram[(self.sp + 1) as usize];
                self.ram[(self.sp + 1) as usize] = xchng_byte;

                xchng_byte = self.l;

                self.l = self.ram[self.sp as usize];
                self.ram[self.sp as usize] = xchng_byte;
                
                self.cycles += 18;
            }

            //CPO addr
            (0xE, 4) => {
                /*
                3 Bytes
                If P not set, CALL addr
                */

                if self.p == false {
                    self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                    self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                    
                    self.sp -= 2;

                    let low_byte = self.ram[self.pc as usize] as u16;
                    let high_byte = self.ram[(self.pc + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.cycles += 17;
                }
                else {
                    self.pc += 2;
                    self.cycles += 11;
                }
                
            }

            //PUSH H
            (0xE, 5) => {
                /*
                1 Byte
                Stores L at (SP - 2), H at (SP - 1), SP - 2
                */

                self.ram[(self.sp - 2) as usize] = self.l;
                self.ram[(self.sp - 1) as usize] = self.h;

                self.sp -= 2;
                self.cycles += 11;
            }

            //ANI D8
            (0xE, 6) => {
                /*
                2 Byte
                A & Immediate affects CY, Z, P, S
                */

                let answer = self.a & self.ram[self.pc as usize];

                self.cy = false; //Resets carry bit
                self.z = (answer & 0xFF) == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;

                self.a = answer;

                self.pc += 1;
                self.cycles += 7;
            }

            //RST 4
            (0xE, 7) => {
                /*
                1 Byte
                CALL $20
                */

                self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                
                self.sp -= 2;

                self.pc = 0x20;
                self.cycles += 11;
            }

            //RPE
            (0xE, 8) => {
                /*
                1 Byte
                If P is set (even parity), RET
                */

                if self.p {
                    let low_byte = self.ram[self.sp as usize] as u16;
                    let high_byte = self.ram[(self.sp + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.sp += 2;
                    self.cycles += 11;
                }
                else {
                    self.cycles += 5;
                    return;
                }
            }

            //PCHL
            (0xE, 9) => {
                /*
                1 Byte
                PC.hi = H; PC.lo = L
                */

                let high_byte = (self.h as u16) << 8;
                let low_byte = self.l as u16;

                self.pc = high_byte | low_byte;
                self.cycles += 5;
            }

            //JPE addr
            (0xE, 0xA) => {
                /*
                3 Byte
                If P  set then PC = addr
                */

                if self.p {
                    let addr = ((self.ram[(self.pc + 1) as usize] as u16) << 8) | ((self.ram[self.pc as usize]) as u16);
                    self.pc = addr;
                    self.cycles += 10;
                } else {
                    self.pc += 2;
                    self.cycles += 10;
                    return;

                }
            }

            //XCHG
            (0xE, 0xB) => {
                /*
                1 Byte
                Exchanges values in H/D and L/E
                */

                let mut xchng_byte = self.h;

                self.h = self.d;
                self.d = xchng_byte;

                xchng_byte = self.l;

                self.l = self.e;
                self.e = xchng_byte;

                self.cycles += 5;
            }

            //CPE addr
            (0xE, 0xC) => {
                /*
                3 Bytes
                If P set, CALL addr
                */

                if self.p {
                    self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                    self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                    
                    self.sp -= 2;

                    let low_byte = self.ram[self.pc as usize] as u16;
                    let high_byte = self.ram[(self.pc + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.cycles += 17;
                }
                else {
                    self.pc += 2;
                    self.cycles += 11;
                }
                
            }
            
            //*CALL
            (0xE, 0xD) => {
                /*
                3 Byte
                (SP-1) = PC.high, (SP-2)= PC.lo, SP = SP-2, PC=addr
                Subroutine Call
                */

                self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                
                self.sp -= 2;

                let low_byte = self.ram[self.pc as usize] as u16;
                let high_byte = self.ram[(self.pc + 1) as usize] as u16;

                self.pc = (high_byte << 8) | low_byte;
                self.cycles += 17;
            }

            //XRI D8
            (0xE, 0xE) => {
                /*
                2 Byte
                Immediate ^ accumulator
                */
                let answer =self.a ^ self.ram[self.pc as usize];
                
                self.cy = false; //Carry bit is reset; 
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = answer.count_ones() % 2 == 0;
                

                self.a = answer;
                self.pc += 1;
                self.cycles += 7;
            }

            //RST 5
            (0xE, 0xF) => {
                /*
                1 Byte
                CALL $28
                */

                self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                
                self.sp -= 2;

                self.pc = 0x28;
                self.cycles += 11;
            }
            
            //RP
            (0xF, 0) => {
                /*
                1 Byte
                If S not set (Positive), RET
                */

                if self.s == false {
                    let low_byte = self.ram[self.sp as usize] as u16;
                    let high_byte = self.ram[(self.sp + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.sp += 2;
                    self.cycles += 11;
                }
                else {
                    self.cycles += 5;
                    return;
                }
            }

            //POP PSW
            (0xF, 1) => {
                /*
                1 Byte
                Flags = (SP), A = (SP + 1), SP + 2
                */

                let flag_val = self.ram[self.sp as usize];
                self.s = (flag_val & 0x80) != 0;
                self.z = (flag_val & 0x40) != 0;
                self.ac = (flag_val & 0x8) != 0;
                self.p = (flag_val & 0x4) != 0;
                self.cy = (flag_val & 0x1) != 0;

                self.a = self.ram[(self.sp + 1) as usize];

                self.sp += 2;
                self.cycles += 10;
            }

            //JP addr
            (0xF, 2) => {
                /*
                3 Byte
                If s not set (Positive) then PC = addr
                */

                if self.s == false {
                    let addr = ((self.ram[(self.pc + 1) as usize] as u16) << 8) | ((self.ram[self.pc as usize]) as u16);
                    self.pc = addr;
                    self.cycles += 10;
                } else {
                    self.pc += 2;
                    self.cycles += 7;
                    return;

                }
            }

            //DI
            (0xF, 3) => {
                self.int_enabled = false;
                self.cycles += 4;
            }  

            //CP addr
            (0xF, 4) => {
                /*
                3 Bytes
                If S not set (Positive), CALL addr
                */

                if self.s == false {
                    self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                    self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                    
                    self.sp -= 2;

                    let low_byte = self.ram[self.pc as usize] as u16;
                    let high_byte = self.ram[(self.pc + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.cycles += 17;
                }
                else {
                    self.pc += 2;
                    self.cycles += 11;
                }
                
            }

            //PUSH PSW
            (0xF, 5) => {
                /*
                1 Byte
                (SP) = Flags, (SP + 1) = A, SP - 2
                */
                
                let mut flag_value:u8 = 0;
                let flag_vec:Vec<bool> = vec![self.s, self.z, false, self.ac, false, self.p, true, self.cy];


                for (i, flag) in flag_vec.iter().enumerate() {
                    if *flag {
                        flag_value |= 1 << (7 - i); //0xb1 is shifted by (7- i) places to set bit, bit it OR'ed to flag value;
                    }
                }
                
                self.ram[(self.sp - 1) as usize] = self.a;

                self.ram[(self.sp - 2) as usize] = flag_value;

                self.sp -= 2;
                self.cycles += 11;
            }

            //ORI D8
            (0xF, 6) => {
                /*
                2 Byte
                Accumulator | Immediate
                */
                let answer =self.a | self.ram[self.pc as usize];
                
                self.cy = false; //Carry bit is reset; 
                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = answer.count_ones() % 2 == 0;
                

                self.a = answer;
                self.pc += 1;
                self.cycles += 7;
            }

            //RST 6
            (0xF, 7) => {
                /*
                1 Byte
                CALL $30
                */

                self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                
                self.sp -= 2;

                self.pc = 0x30;
                self.cycles += 11;
            }

            //RM
            (0xF, 8) => {
                /*
                1 Byte
                If S is set, RET
                */

                if self.s {
                    let low_byte = self.ram[self.sp as usize] as u16;
                    let high_byte = self.ram[(self.sp + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.sp += 2;
                    self.cycles += 11;
                }
                else {
                    self.cycles += 5;
                    return;
                }
            }

            //SPHL
            (0xF, 9) => {
                /*
                1 Byte
                SP = HL
                */

                let hl_16 = ((self.h as u16) << 8) | self.l as u16;

                self.sp = hl_16;
                self.cycles += 5;
            }

            //JM addr
            (0xF, 0xA) => {
                /*
                3 Byte
                If S set (Minus) then PC = addr
                */

                if self.s {
                    let addr = ((self.ram[(self.pc + 1) as usize] as u16) << 8) | ((self.ram[self.pc as usize]) as u16);
                    self.pc = addr;
                    self.cycles += 10;
                } else {
                    self.pc += 2;
                    self.cycles += 10;
                    return;

                }
            }

            //EI
            (0xF, 0xB) => {
                self.int_enabled = true;
                self.cycles += 4;
            }

            //CM addr
            (0xF, 0xC) => {
                /*
                3 Bytes
                If S is set (Minus), CALL addr
                */

                if self.s {
                    self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                    self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                    
                    self.sp -= 2;

                    let low_byte = self.ram[self.pc as usize] as u16;
                    let high_byte = self.ram[(self.pc + 1) as usize] as u16;

                    self.pc = (high_byte << 8) | low_byte;
                    self.cycles += 17;
                }
                else {
                    self.pc += 2;
                    self.cycles += 11;
                }
                
            }

            //*CALL
            (0xF, 0xD) => {
                /*
                3 Byte
                (SP-1) = PC.high, (SP-2)= PC.lo, SP = SP-2, PC=addr
                Subroutine Call
                */

                self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                
                self.sp -= 2;

                let low_byte = self.ram[self.pc as usize] as u16;
                let high_byte = self.ram[(self.pc + 1) as usize] as u16;

                self.pc = (high_byte << 8) | low_byte;
                self.cycles += 17;
            }
          
            //CPI D8
            (0xF, 0xE) => {
                /*
                2 Bytes
                Sets Flags based on comparison of A and data
                */
                let immediate = self.ram[self.pc as usize];
                let (answer, carry) = self.a.overflowing_sub(immediate);

                self.z = answer == 0;
                self.s = (answer & 0x80) != 0;
                self.p = answer.count_ones() % 2 == 0;
                self.cy = carry;
                self.ac = (self.a & 0x0F) < (immediate & 0x0F);

                self.pc += 1;
                self.cycles += 7;
            }

            //RST 7
            (0xF, 0xF) => {
                /*
                1 Byte
                CALL $38
                */

                self.ram[(self.sp - 1) as usize] = ((self.pc + 2) >> 8) as u8;
                self.ram[(self.sp - 2) as usize] = (self.pc + 2) as u8;
                
                self.sp -= 2;

                self.pc = 0x38;
                self.cycles += 11;
            }

            (_, _) => {
                
                #[cfg(feature = "debug")]
                write_to_file(&self.ram).unwrap();
                
                panic!("Unimplemented opcode: {:02x}", op)},
        }
    }
}

#[cfg(feature = "debug")]
fn write_to_file(data: &[u8]) -> std::io::Result<()> {
    let mut file = File::create("ram_output.txt")?;
    file.write_all(data)?;
    Ok(())
}
