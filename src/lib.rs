
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
        //Fetch & Decode
        let op:u8 = self.fetch();
        
        //Execute
        self.execute(op);
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
                self.pc += 2;

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
                let byte_2 = self.ram[(self.pc) as usize];

                self.b = byte_2;

                self.pc += 1;

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
                let byte_2 = self.ram[(self.pc) as usize];

                self.c = byte_2;

                self.pc += 1;

            }

            //RRC
            (0, 0xF) => {
                /*
                1 Byte
                Rotate Accumulator Right, sets CY to LMB shits A by 1 and concats CY to A
                */
                self.cy = (self.a & 0x01) != 0;

                self.a = self.a >> 1;

                self.a |= (self.cy as u8) >> 7;

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
                self.pc += 2;

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
                let byte_2 = self.ram[(self.pc) as usize];

                self.d = byte_2;

                self.pc += 1;

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
                let byte_2 = self.ram[(self.pc) as usize];

                self.e = byte_2;

                self.pc += 1;

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
                3 Byte instruction, (OP/L-Byte/H-Byte)
                3 MCycles Op Fetch/Mem Read/Mem Read
                */
                let low_byte = self.ram[self.pc as usize] as u8;
                let high_byte = self.ram[(self.pc + 1) as usize];
                
                self.l = low_byte;
                self.h = high_byte;
                self.pc += 2;

            }

            //SHLD adr
            (2, 2) => {
                /*
                3 Byte
                Stores L at (Byte 2) and H at (Byte 2) + 1
                */

                let addr = self.ram[self.pc as usize] as usize;

                self.ram[addr] = self.l;
                if addr + 1  <= RAM_SIZE {
                    self.ram[addr + 1] = self.h;
                    self.pc += 2;
                } 
                else {
                panic!("SHLD Acessing Nonexistant Memory Addr!")};
                
            }

            //INX H
            (2,3) => {
                /*
                1 Byte
                Increments H and L by one, does not affect flags
                */
                self.h = self.h.wrapping_add(1);
                self.l = self.l.wrapping_add(1);

            }

            //INR H
            (2, 4) => {
                /*
                1 Byte
                Increments H, flags = Z, S, P, AC
                */
                let answer:u8 = self.h.wrapping_add(1);
                self.z = (answer & 0xFF) == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;
                self.ac = answer & 0xF == 0;

                self.h = answer as u8;
            }

            //DCR H
            (2,5) => {
                /*
                1 Byte
                Decrements (HL), flags = Z, S, P, AC
                */

                let high_byte = self.h as u16;
                let low_byte = self.l as u16;
                let addr = (high_byte << 8) | low_byte;

                let answer = self.ram[addr as usize];

                let answer = answer.wrapping_sub(1);
                self.z = (answer & 0xFF) == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;
                self.ac = answer & 0xF == 0;
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

            }

            //DAA
            (2, 7) => {
                /*
                1 Byte
                8 Bit accumulator is adjusted to form two 4 bit binary-coded-decimal digits
                Method as for i8080 manual
                */
                
                let mut ls_nibble = self.a & 0x0F;
                let mut ms_nibble = self.a >> 4;

                if ls_nibble > 9 || self.cy { //ls_nibble is checked to see if greater than 9 or CY flag is set
                    ls_nibble = ls_nibble.wrapping_add(6); // 6 is added to ls_nibble
                    self.cy = ls_nibble > 15; // CY flag is set if carry is present
                }

                if ms_nibble > 9 || self.ac { // ms_nibble is checked to see if greater than 9 or AC flag is set
                    ms_nibble = ms_nibble.wrapping_add(6); // 6 is added to ms_nibble
                    self.ac = ms_nibble > 15; // AC flag is set if carry is present
                }

                self.a = (ms_nibble << 4) | ls_nibble; // Nibbles are merged after computing

                // Other Flags are set
                self.z = (self.a & 0xFF) == 0;
                self.s = (self.a & 0x80) != 0;
                self.p = (self.a.count_ones() % 2) == 0;

            }

            //*NOP
            (2, 8) => return,

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
            }

            //LHLD adr
            (2, 0xA) => {
                /*
                3 Byte
                Stores (Byte 2) at L and (Byte 3) at H
                */

                self.l = self.ram[self.pc as usize];
                self.h = self.ram[(self.pc + 1) as usize];

                self.pc += 2;
            }

            //DCX H
            (2, 0xB) => {
                /*
                1 Byte
                Decrements Register Pair
                */
                self.h = self.h.wrapping_sub(1);
                self.l = self.l.wrapping_sub(1);
            }

            //INR L
            (2, 0xC) => {
                /*
                1 Byte
                Increments H, flags = Z, S, P, AC
                */
                let answer:u8 = self.l.wrapping_add(1);
                self.z = (answer & 0xFF) == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;
                self.ac = answer & 0xF == 0;

                self.l = answer;
            }

            //MVI L, D8
            (2, 0xD) => {
                /*
                2 Byte
                Moves (byte 2) to L                
                */
                let byte_2 = self.ram[(self.pc) as usize];

                self.l = byte_2;

                self.pc += 1;

            }

            //CMA
            (2, 0xF) => {
                /*
                1 Byte
                Returns Ones compliment of accumulator
                */
                self.a = !self.a;
            }

            //*NOP
            (3, 0) => return,

            //LXI , D16
            (3,1) => {
                /*
                3 Byte instruction, (OP/L-Byte/H-Byte)
                3 MCycles Op Fetch/Mem Read/Mem Read
                */
                let low_byte = self.ram[self.pc as usize] as u16;
                let high_byte = self.ram[(self.pc + 1) as usize] as u16;

                self.sp = (high_byte << 8) | low_byte;
                self.pc += 2;

            }

            //STA , A16
            (3,2) => {
                /*
                3 Byte instruction, (OP/L-Byte/H-Byte)
                3 MCycles Op Fetch/Mem Read/Mem Read
                Stores A in two byte addr
                */
                let low_byte = self.ram[self.pc as usize] as u16;
                let high_byte = self.ram[(self.pc + 1) as usize] as u16;
                let addr = ((high_byte << 8) | low_byte) as usize;

                self.ram[addr] = self.a;
                self.pc += 2;

            }

            /* TODO: IMPLEMENT MISSING OP CODES */

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

            }

            //LDA adr
            (3, 0xA) => {
                /*
                1 Byte
                Loads addr from A
                */
                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);
                self.ram[addr as usize] = self.a; 
            }

            //MVI A, D8
            (2, 0xE) => {
                /*
                2 Byte
                Moves (byte 2) to L                
                */
                let byte_2 = self.ram[(self.pc) as usize];

                self.a = byte_2;

                self.pc += 1;

            }

            //MOV D, M
            (5, 6) => {
                /*
                1 Byte
                Moves data from M (HL) to D
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.d = self.ram[addr as usize];
            }

            //MOV E, M
            (5, 0xE) => {
                /*
                1 Byte
                Moves data from M (HL) to E
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.e = self.ram[addr as usize];
            }

            //MOV H, M
            (6, 6) => {
                /*
                1 Byte
                Moves data from M (HL) to H
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.h = self.ram[addr as usize];
            }

            //MOV L, A
            (6, 0xF) => {
                /*
                1 Byte
                Moves data from A to L
                */

                self.l = self.a;
            }

            //MOV M, A
            (7, 7) => {
                /*
                1 Byte
                Moves data from M (HL) to H
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.a = self.ram[addr as usize];
            }

            //MOV A, D
            (7, 0xA) => {
                /*
                1 Byte
                Moves data from D to A
                */

                self.a = self.d;
            }

            //MOV A, E
            (7, 0xB) => {
                /*
                1 Byte
                Moves data from E to A
                */

                self.a = self.e;
            }

            //MOV A, H
            (7, 0xC) => {
                /*
                1 Byte
                Moves data from H to A
                */

                self.a = self.h;
            }

            //MOV A, M
            (7, 0xE) => {
                /*
                1 Byte
                Moves data from M to A
                */

                let addr:u16 = ((self.h as u16) << 8) | (self.l as u16);

                self.a = self.ram[addr as usize];
            }

            //ANA A
            (0xA, 7) => {
                /*
                1 Byte
                A & A affects CY, Z, S, P, AC
                */

                let answer = self.a & self.a;

                self.cy = false; //Resets carry bit
                self.z = (answer & 0xFF) == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;
                self.ac = answer & 0xF == 0;

                self.a = answer;

            }

            //ANA A
            (0xA, 0xF) => {
                /*
                1 Byte
                A ^ A affects CY, Z, S, P, AC
                */

                let answer = self.a ^ self.a;

                self.cy = false; //Resets carry bit
                self.z = (answer & 0xFF) == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;
                self.ac = answer & 0xF == 0;

                self.a = answer;

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
            }

            //JNZ adr
            (0xC, 2) => {
                /*
                3 Byte
                If Z not set then PC = addr
                */

                if self.z == false {
                    let addr:u16 = (((self.pc + 1) as u16) << 8) | (self.pc as u16);
                    self.pc = addr;
                } else {
                    return;
                }
            }

            //JNZ adr
            (0xC, 3) => {
                /*
                3 Byte
                PC = Addr
                */
                let addr:u16 = (((self.pc + 1) as u16) << 8) | (self.pc as u16);

                self.pc = addr;
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
            }

            //ADI
            (0xC, 6) => {
                /*
                2 Byte
                Adds immediate to accumulator
                */
                let answer = self.ram[self.pc as usize];

                self.z = (answer & 0xFF) == 0;
                self.s = (answer & 0x80) != 0;
                self.p = (answer.count_ones() % 2) == 0;
                self.ac = answer & 0xF == 0;
                self.cy = answer & 0xF == 0;

                self.a = answer;
                self.pc += 1;
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
            }

            //CALL adr
            (0xC, 0xD) => {
                /*
                3 Byte
                (SP-1) = PC.high, (SP-2)= PC.lo, SP = SP-2, PC=addr
                Subroutine Call
                */

                self.ram[(self.sp - 1) as usize] = ((self.pc - 1) >> 8) as u8;
                self.ram[(self.sp - 2) as usize] = (self.pc - 1) as u8;
                
                self.sp -= 2;

                let low_byte = self.ram[self.pc as usize] as u16;
                let high_byte = self.ram[(self.pc + 1) as usize] as u16;

                self.pc = (high_byte << 8) | low_byte;
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
            }

            //OUT D8
            (0xD, 3) => {
                unimplemented!("Output Attempted {}", op)
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
            }

            //POP PSW
            (0xF, 1) => {
                /*
                1 Byte
                Flags = (SP), A = (SP + 1), SP + 2
                */

                let flag_val = self.ram[self.sp as usize];
                self.s = flag_val & 0x80 == 1;
                self.z = flag_val & 0x40 == 1;
                self.ac = flag_val & 0x8 == 1;
                self.p = flag_val & 0x4 == 1;
                self.cy = flag_val & 0x1 == 1;

                self.a = self.ram[(self.sp + 1) as usize];

                self.sp += 2;

            }

            //PUSH PSW
            (0xF, 5) => {
                /*
                1 Byte
                (SP) = Flags, (SP + 1) = A, SP - 2
                */

                let mut flag_value:u8 = 0;
                flag_value = (flag_value | (self.s as u8)) << 1; //Sign bit
                flag_value = (flag_value | (self.z as u8)) << 1; //Zero bit
                flag_value <<= 1;
                flag_value = (flag_value | (self.ac as u8)) << 1; //Aux Carry bit
                flag_value <<= 1;
                flag_value = (flag_value | (self.p as u8)) << 1; //Parity bit
                flag_value = (flag_value | 1) << 1;
                flag_value = (flag_value | (self.cy as u8)) << 1; //Carry bit

                self.ram[self.sp as usize] = flag_value;

                self.ram[(self.sp + 1) as usize] = self.a;

                self.sp -= 2;
            }

            //EI
            (0xF, 0xB) => {
                unimplemented!("Enabling interrupts attempted: {}", op)
            }

            //CPI D8
            (0xF, 0xE) => {
                /*
                2 Bytes
                Sets Flags based on comparison of A and data
                */
                let data = self.ram[self.pc as usize];
                let answer = self.a.overflowing_sub(data);

                self.z = answer.0 == self.a;
                self.s = (answer.0 & 0x80) != 0;
                self.p = (answer.0.count_ones() % 2) == 0;
                self.ac = answer.1;

                //If A and data signs are same carry bit is set normally, if different carry bit is set invertedly
                self.cy = if ((data >> 7) & (self.a >> 7)) != 0 {answer.1 != true} else {answer.1 == true};
                
            }


            (_, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }
}