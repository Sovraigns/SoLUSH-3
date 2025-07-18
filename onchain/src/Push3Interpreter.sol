// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/console.sol";

/**
 * @title Push3Interpreter
 * @dev Demonstrates a minimal Push3-like interpreter that uses
 *      a token-based bytecode approach.
 */
contract Push3Interpreter {
    // ReadUint`numBits` out of the code range.
    error ReadUintOutOfRange(uint256 numBits);
    // -----------------------------------------------------
    // 0. CONSTANTS
    // -----------------------------------------------------
    uint8 internal constant OPCODE_INTEGER_OFFSET = uint8(OpCode.INTEGER_PLUS);
    uint8 internal constant OPCODE_BOOL_OFFSET = uint8(OpCode.BOOL_DUP);
    uint8 internal constant OPCODE_LAST = uint8(OpCode.IF_ELSE);

    // -----------------------------------------------------
    // 1. ENUMS
    // -----------------------------------------------------
    enum CodeTag {
        NO_TAG,       // 0
        INSTRUCTION,  // 1
        INT_LITERAL,  // 2
        BOOL_LITERAL, // 3
        SUBLIST       // 4
    }

    enum OpCode {
        NOOP,            // 0
        NOOP_1,          // 1 - empty for INSTRUCTION
        NOOP_2,          // 2 - empty for INT_LITERAL
        NOOP_3,          // 3 - empty for BOOL_LITERAL
        NOOP_4,          // 4 - empty for SUBLIST
        INTEGER_PLUS,    // 5 - OPCODE_INTEGER_OFFSET
        INTEGER_MINUS,   // OPCODE_INTEGER_OFFSET + 1
        INTEGER_MULT,    // OPCODE_INTEGER_OFFSET + 2
        INTEGER_DUP,     // OPCODE_INTEGER_OFFSET + 3
        INTEGER_POP,     // OPCODE_INTEGER_OFFSET + 4
        BOOL_DUP,        // 10 - OPCODE_BOOL_OFFSET
        BOOL_POP,        // OPCODE_BOOL_OFFSET + 1
        BOOL_SWAP,       // OPCODE_BOOL_OFFSET + 2
        BOOL_FLUSH,      // OPCODE_BOOL_OFFSET + 3
        BOOL_STACKDEPTH, // OPCODE_BOOL_OFFSET + 4
        BOOL_NOT,        // OPCODE_BOOL_OFFSET + 5
        BOOL_AND,        // OPCODE_BOOL_OFFSET + 6
        BOOL_OR,         // OPCODE_BOOL_OFFSET + 7
        BOOL_EQ,         // OPCODE_BOOL_OFFSET + 8
        BOOL_FROMFLOAT,  // OPCODE_BOOL_OFFSET + 9
        BOOL_FROMINTEGER,// OPCODE_BOOL_OFFSET + 10
        BOOL_ROT,        // OPCODE_BOOL_OFFSET + 11
        BOOL_SHOVE,      // OPCODE_BOOL_OFFSET + 12
        BOOL_YANK,       // OPCODE_BOOL_OFFSET + 13
        BOOL_YANKDUP,    // OPCODE_BOOL_OFFSET + 14
        BOOL_DEFINE,     // OPCODE_BOOL_OFFSET + 15
        BOOL_RAND,       // OPCODE_BOOL_OFFSET + 16 (last bool opcode)
        
        // Comparison operations (0x20-0x2F range)
        INTEGER_GT,      // 0x20 - Greater than
        INTEGER_LT,      // 0x21 - Less than  
        INTEGER_EQ,      // 0x22 - Equal
        INTEGER_NE,      // 0x23 - Not equal
        INTEGER_GE,      // 0x24 - Greater equal
        INTEGER_LE,      // 0x25 - Less equal
        
        // Mathematical functions (0x30-0x3F range)
        INTEGER_SIN,     // 0x30 - Sine
        INTEGER_COS,     // 0x31 - Cosine
        INTEGER_SQRT,    // 0x32 - Square root
        INTEGER_ABS,     // 0x33 - Absolute value
        INTEGER_MOD,     // 0x34 - Modulo
        INTEGER_POW,     // 0x35 - Power
        
        // Constants (0x40-0x4F range)
        CONST_PI,        // 0x40 - π
        CONST_E,         // 0x41 - e
        CONST_RAND,      // 0x42 - Random [0,1000)
        
        // Type conversions (0x50-0x5F range)
        BOOL_TO_INT,     // 0x50 - Bool to int
        INT_TO_BOOL,     // 0x51 - Int to bool
        
        // Conditional operations (0x60-0x6F range)
        IF_THEN,         // 0x60 - If-then
        IF_ELSE          // 0x61 - If-else
    }

    // -----------------------------------------------------
    // 2. DESCRIPTOR BIT LAYOUT
    // -----------------------------------------------------
    // [255..248] : tag (8 bits)
    // [247..216] : offset (32 bits)  (for SUBLIST)
    // [215..184] : length (32 bits)  (for SUBLIST)
    // [183..0]   : leftover bits     (opcode or literal data)

    function getTag(uint256 desc) internal pure returns (CodeTag) {
        return CodeTag(uint8(desc >> 248));
    }

    function getOffset(uint256 desc) internal pure returns (uint32) {
        return uint32(desc >> 216);
    }

    function getLength(uint256 desc) internal pure returns (uint32) {
        return uint32(desc >> 184);
    }

    function getLow184(uint256 desc) internal pure returns (uint256) {
        return desc & ((1 << 184) - 1);
    }

    // Build a descriptor
    function makeDescriptor(
        CodeTag tag,
        uint32 offset,
        uint32 length,
        uint256 low184
    )
        public
        pure
        returns (uint256)
    {
        require(low184 < (1 << 184), "low184 out of range");
        return 
            (uint256(uint8(tag)) << 248) |
            (uint256(offset) << 216) |
            (uint256(length) << 184) |
            low184;
    }

    function makeInstruction(OpCode op) internal pure returns (uint256) {
        return makeDescriptor(CodeTag.INSTRUCTION, 0, 0, uint256(uint8(op)));
    }

    function getOpCode(uint256 desc) internal pure returns (OpCode) {
        return OpCode(uint8(desc & 0xFF));
    }

    function makeIntLiteral(int32 val) internal pure returns (uint256) {
        // store 32-bit integer in leftover
        uint256 masked = uint256(uint32(val));
        return makeDescriptor(CodeTag.INT_LITERAL, 0, 0, masked);
    }

    function makeBoolLiteral(bool val) internal pure returns (uint256) {
        uint256 masked;
        if (val) masked = uint256(1);
        return makeDescriptor(CodeTag.BOOL_LITERAL, 0, 0, masked);
    }

    function extractInt32(uint256 desc) internal pure returns (int32) {
        // read the low 32 bits
        return int32(uint32(getLow184(desc)));
    }

    function extractBool(uint256 desc) internal pure returns (bool) {
        return desc & 1 == 1;
    }

    // -----------------------------------------------------
    // 3. HELPER READS
    // -----------------------------------------------------

    /**
     * @dev Read `x` bytes from `code` at `start` as uint.
     */
    function readUint(bytes calldata code, uint32 start, uint256 numBytes) internal pure returns (uint256 word) {
        uint256 numBits = numBytes * 8;
        if (start + numBytes > code.length) revert ReadUintOutOfRange(numBits);
        assembly {
            let buf := mload(0x40) // free memory
            // copy exactly x bytes from code into buf
            calldatacopy(buf, add(code.offset, start), numBytes)
            word := mload(buf)
        }
        word >>= 256 - numBits;
    }

    /**
     * @dev Read 4 bytes from `code` at `start` as uint32.
     */
    function readUint32(bytes calldata code, uint32 start) internal pure returns (uint32 val) {
        val = uint32(readUint(code, start, 4));
    }

    /**
     * @dev Read 2 bytes from `code` at `start` as uint16.
     */
    function readUint16(bytes calldata code, uint32 start) internal pure returns (uint16 val) {
        val = uint16(readUint(code, start, 2));
    }

    /**
     * @dev Read 1 byte from `code` at `start` as uint8.
     */
    function readUint8(bytes calldata code, uint32 start) internal pure returns (uint8 val) {
        val = uint8(readUint(code, start, 1));
    }

    /**
     * @dev Read 1 byte from `code` at `start` as bool.
     */
    function readBool(bytes calldata code, uint32 start) internal pure returns (bool val) {
        val = readUint(code, start, 1) & 1 == 1;
    }

    // -----------------------------------------------------
    // 4. SUBLIST PARSING
    // -----------------------------------------------------
    /**
     * Token format is combination of ordered CodeTag and OpCode:
     *   0x00 => NOOP/NO_TAG
     *   0x01 => INSTRUCTION => read next 1 byte => OpCode
     *   0x02 => INT_LITERAL => read next 4 bytes => int32
     *   0x03 => BOOL_LITERAL => read next 1 byte => bool
     *   0x04 => SUBLIST => read next 2 bytes => subLen => parse that
     *   0x05 => INTEGER_PLUS
     *   0x06 => INTEGER_MINUS
     *   ... opcode list with type offsets
     */
    function parseSublist(bytes calldata code, uint32 off, uint32 len)
        internal
        pure
        returns (uint256[] memory descs)
    {
        uint256[] memory temp = new uint256[](len);
        uint256 count = 0;
        uint32 endPos = off + len;
        uint32 cur = off;

        while (cur < endPos) {
            if (cur >= code.length) {
                break;
            }

            uint8 tokenType = uint8(code[cur]);
            cur++;

            if (tokenType == 0x00) {
                // NOOP
                temp[count] = makeInstruction(OpCode.NOOP);
                count++;
            }
            else if (tokenType == 0x01) {
                // INSTRUCTION
                if (cur + 1 <= endPos && (cur + 1) <= code.length) {
                    uint8 opcode = readUint8(code,cur);
                    cur += 1;
                    if (opcode < OPCODE_INTEGER_OFFSET || opcode > OPCODE_LAST) opcode = uint8(0); // NOOP
                    temp[count] = makeInstruction(OpCode(opcode));
                    count++;
                } else {
                    break;
                }
            }
            else if (tokenType == 0x02) {
                // INT_LITERAL => 4 bytes
                if (cur + 4 <= endPos && (cur + 4) <= code.length) {
                    uint32 raw = readUint32(code, cur);
                    cur += 4;
                    int32 val = int32(raw);
                    temp[count] = makeIntLiteral(val);
                    count++;
                } else {
                    break;
                }
            }
            else if (tokenType == 0x03) {
                // BOOL_LITERAL
                if (cur + 1 <= endPos && (cur + 1) <= code.length) {
                    bool val = readBool(code, cur);
                    cur += 1;
                    temp[count] = makeBoolLiteral(val);
                    count++;
                } else {
                    break;
                }
            }
            else if (tokenType == 0x04) {
                // SUBLIST => read 2 bytes => length
                if (cur + 2 <= endPos && (cur + 2) <= code.length) {
                    uint16 subLen = readUint16(code, cur);
                    cur += 2;
                    if (cur + subLen <= endPos && (cur + subLen) <= code.length) {
                        // create descriptor
                        temp[count] = makeDescriptor(CodeTag.SUBLIST, cur, subLen, 0);
                        count++;
                        // skip over that subLen chunk
                        cur += subLen;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            else if (tokenType <= OPCODE_LAST) {
                // OPCODE
                temp[count] = makeInstruction(OpCode(tokenType));
                count++;
            }
            else {
                // unknown => NOOP
                temp[count] = makeInstruction(OpCode.NOOP);
                count++;
            }
        }

        descs = new uint256[](count);
        for (uint256 i = 0; i < count; i++) {
            descs[i] = temp[i];
        }
    }

    // -----------------------------------------------------
    // 5. MATHEMATICAL HELPER FUNCTIONS
    // -----------------------------------------------------
    
    /// Simple integer square root using binary search
    function _sqrt(uint256 x) internal pure returns (uint256) {
        if (x == 0) return 0;
        if (x < 4) return 1;
        
        uint256 z = x;
        uint256 y = x / 2 + 1;
        while (y < z) {
            z = y;
            y = (x / y + y) / 2;
        }
        return z;
    }
    
    /// Simple integer power function
    function _pow(int256 base, int256 exponent) internal pure returns (int256) {
        if (exponent < 0) return 0;
        if (exponent == 0) return 1;
        if (base == 0) return 0;
        
        int256 result = 1;
        int256 b = base;
        uint256 exp = uint256(exponent);
        
        while (exp > 0) {
            if (exp % 2 == 1) {
                result = result * b;
            }
            b = b * b;
            exp = exp / 2;
        }
        return result;
    }
    
    /// Simplified sine approximation for integers (input in degrees * 10)
    function _sin(int256 x) internal pure returns (int256) {
        // Normalize to [0, 3600) (360 degrees * 10)
        x = x % 3600;
        if (x < 0) x += 3600;
        
        // Simple lookup table for key angles
        if (x == 0) return 0;        // 0°
        if (x == 900) return 1000;   // 90°
        if (x == 1800) return 0;     // 180°
        if (x == 2700) return -1000; // 270°
        
        // Linear approximation for other values
        if (x <= 900) {
            return (1000 * x) / 900;
        } else if (x <= 1800) {
            return (1000 * (1800 - x)) / 900;
        } else if (x <= 2700) {
            return -(1000 * (x - 1800)) / 900;
        } else {
            return -(1000 * (3600 - x)) / 900;
        }
    }
    
    /// Simplified cosine approximation for integers (input in degrees * 10)  
    function _cos(int256 x) internal pure returns (int256) {
        // cos(x) = sin(x + 90°)
        return _sin(x + 900);
    }
    
    // -----------------------------------------------------
    // 6. EXTENDED OPCODE HANDLER
    // -----------------------------------------------------
    
    function handleExtendedOpcodes(
        OpCode op,
        int256[] memory intStack,
        bool[] memory boolStack,
        uint256[] memory execStack,
        uint256 intTop,
        uint256 boolTop,
        uint256 execTop
    ) internal view returns (uint256, uint256, uint256) {
        // Mathematical functions
        if (op == OpCode.INTEGER_SIN) {
            if (intTop >= 1) {
                int256 a = intStack[intTop - 1];
                intStack[intTop - 1] = _sin(a);
            }
        }
        else if (op == OpCode.INTEGER_COS) {
            if (intTop >= 1) {
                int256 a = intStack[intTop - 1];
                intStack[intTop - 1] = _cos(a);
            }
        }
        else if (op == OpCode.INTEGER_SQRT) {
            if (intTop >= 1) {
                int256 a = intStack[intTop - 1];
                if (a >= 0) {
                    intStack[intTop - 1] = int256(_sqrt(uint256(a)));
                } else {
                    intStack[intTop - 1] = 0;
                }
            }
        }
        else if (op == OpCode.INTEGER_ABS) {
            if (intTop >= 1) {
                int256 a = intStack[intTop - 1];
                intStack[intTop - 1] = a >= 0 ? a : -a;
            }
        }
        else if (op == OpCode.INTEGER_MOD) {
            if (intTop >= 2) {
                int256 a = intStack[intTop - 1];
                int256 b = intStack[intTop - 2];
                intTop -= 2;
                if (a != 0) {
                    intStack[intTop] = b % a;
                } else {
                    intStack[intTop] = 0;
                }
                intTop++;
            }
        }
        else if (op == OpCode.INTEGER_POW) {
            if (intTop >= 2) {
                int256 a = intStack[intTop - 1]; // exponent
                int256 b = intStack[intTop - 2]; // base
                intTop -= 2;
                intStack[intTop] = _pow(b, a);
                intTop++;
            }
        }
        // Constants
        else if (op == OpCode.CONST_PI) {
            intStack[intTop] = 3141; // π * 1000 for precision
            intTop++;
        }
        else if (op == OpCode.CONST_E) {
            intStack[intTop] = 2718; // e * 1000 for precision
            intTop++;
        }
        else if (op == OpCode.CONST_RAND) {
            uint256 randomNum = uint256(keccak256(abi.encodePacked(block.prevrandao, block.timestamp, msg.sender, intTop)));
            intStack[intTop] = int256(randomNum % 1000); // Random [0,999]
            intTop++;
        }
        // Type conversions
        else if (op == OpCode.BOOL_TO_INT) {
            if (boolTop >= 1) {
                bool val = boolStack[boolTop - 1];
                boolTop--;
                intStack[intTop] = val ? int256(1) : int256(0);
                intTop++;
            }
        }
        else if (op == OpCode.INT_TO_BOOL) {
            if (intTop >= 1) {
                int256 val = intStack[intTop - 1];
                intTop--;
                boolStack[boolTop] = val != 0;
                boolTop++;
            }
        }
        // Conditional operations
        else if (op == OpCode.IF_THEN) {
            if (boolTop >= 1 && execTop >= 1) {
                bool condition = boolStack[boolTop - 1];
                boolTop--;
                if (!condition) {
                    // Skip next instruction by not pushing it back
                    execTop--;
                }
            }
        }
        else if (op == OpCode.IF_ELSE) {
            if (boolTop >= 1 && execTop >= 2) {
                bool condition = boolStack[boolTop - 1];
                boolTop--;
                uint256 thenItem = execStack[execTop - 1];
                uint256 elseItem = execStack[execTop - 2];
                execTop -= 2;
                if (condition) {
                    execStack[execTop] = thenItem;
                } else {
                    execStack[execTop] = elseItem;
                }
                execTop++;
            }
        }
        
        return (intTop, boolTop, execTop);
    }
    
    // -----------------------------------------------------
    // 7. MAIN INTERPRETER
    // -----------------------------------------------------
    function runInterpreter(
        bytes calldata code, // non-empty - genetic agent (DNA)
        uint256[] calldata initCodeStack, // empty
        uint256[] calldata initExecStack, // descriptors
        int256[] calldata initIntStack, // 32bit word
        bool[] calldata initBoolStack // bool array
    )
        external
        view
        returns (
            uint256[] memory finalCodeStack,
            uint256[] memory finalExecStack,
            int256[]  memory finalIntStack,
            bool[] memory finalBoolStack
        )
    {
        // A) CODE STACK
        uint256[] memory codeStack = new uint256[](initCodeStack.length + 256);
        uint256 codeTop = initCodeStack.length;
        for (uint256 i = 0; i < initCodeStack.length; i++) {
            codeStack[i] = initCodeStack[i];
        }

        // B) EXEC STACK
        uint256[] memory execStack = new uint256[](initExecStack.length + 256);
        uint256 execTop = initExecStack.length;
        for (uint256 e = 0; e < initExecStack.length; e++) {
            execStack[e] = initExecStack[e];
        }

        // C) INT STACK
        int256[] memory intStack = new int256[](initIntStack.length + 256);
        uint256 intTop = initIntStack.length;
        for (uint256 k = 0; k < initIntStack.length; k++) {
            intStack[k] = initIntStack[k];
        }

        // D) BOOL STACK
        bool[] memory boolStack = new bool[](initBoolStack.length + 256);
        uint256 boolTop = initBoolStack.length;
        for (uint256 k = 0; k < initBoolStack.length; k++) {
            boolStack[k] = initBoolStack[k];
        }

        // D) MAIN LOOP
        while (execTop > 0) {
            uint256 topDesc = execStack[execTop - 1];
            execTop--;

            CodeTag tag = getTag(topDesc);

            if (tag == CodeTag.INSTRUCTION) {
                OpCode op = getOpCode(topDesc);
                if (uint8(op) < OPCODE_INTEGER_OFFSET) {
                    // NOOP, do nothing
                }
                else if (uint8(op) < OPCODE_BOOL_OFFSET || uint8(op) >= 0x20) {
                    // INTEGER OPCODES and new extended opcodes
                    if (op == OpCode.INTEGER_PLUS) {
                        // pop top 2 => sum
                        if (intTop >= 2) {
                            int256 a = intStack[intTop - 1];
                            int256 b = intStack[intTop - 2];
                            intTop -= 2;
                            intStack[intTop] = b + a;
                            intTop++;
                        }
                    }
                    else if (op == OpCode.INTEGER_MINUS) {
                        // pop top 2 => minus
                        if (intTop >= 2) {
                            int256 a = intStack[intTop - 1];
                            int256 b = intStack[intTop - 2];
                            intTop -= 2;
                            intStack[intTop] = b - a;
                            intTop++;
                        }
                    }
                    else if (op == OpCode.INTEGER_MULT) {
                        // pop top 2 => minus
                        if (intTop >= 2) {
                            int256 a = intStack[intTop - 1];
                            int256 b = intStack[intTop - 2];
                            intTop -= 2;
                            intStack[intTop] = b * a;
                            intTop++;
                        }
                    }
                    else if (op == OpCode.INTEGER_DUP) {
                        if (intTop >= 1) {
                            int256 a = intStack[intTop - 1];
                            intStack[intTop] = a;
                            intTop++;
                        }
                    }
                    else if (op == OpCode.INTEGER_POP) {
                        if (intTop >= 1) {
                            intTop -= 1;
                        }
                    }
                    // Comparison operations - result pushed to bool stack
                    else if (op == OpCode.INTEGER_GT) {
                        if (intTop >= 2) {
                            int256 a = intStack[intTop - 1];
                            int256 b = intStack[intTop - 2];
                            intTop -= 2;
                            boolStack[boolTop] = b > a;
                            boolTop++;
                        }
                    }
                    else if (op == OpCode.INTEGER_LT) {
                        if (intTop >= 2) {
                            int256 a = intStack[intTop - 1];
                            int256 b = intStack[intTop - 2];
                            intTop -= 2;
                            boolStack[boolTop] = b < a;
                            boolTop++;
                        }
                    }
                    else if (op == OpCode.INTEGER_EQ) {
                        if (intTop >= 2) {
                            int256 a = intStack[intTop - 1];
                            int256 b = intStack[intTop - 2];
                            intTop -= 2;
                            boolStack[boolTop] = b == a;
                            boolTop++;
                        }
                    }
                    else if (op == OpCode.INTEGER_NE) {
                        if (intTop >= 2) {
                            int256 a = intStack[intTop - 1];
                            int256 b = intStack[intTop - 2];
                            intTop -= 2;
                            boolStack[boolTop] = b != a;
                            boolTop++;
                        }
                    }
                    else if (op == OpCode.INTEGER_GE) {
                        if (intTop >= 2) {
                            int256 a = intStack[intTop - 1];
                            int256 b = intStack[intTop - 2];
                            intTop -= 2;
                            boolStack[boolTop] = b >= a;
                            boolTop++;
                        }
                    }
                    else if (op == OpCode.INTEGER_LE) {
                        if (intTop >= 2) {
                            int256 a = intStack[intTop - 1];
                            int256 b = intStack[intTop - 2];
                            intTop -= 2;
                            boolStack[boolTop] = b <= a;
                            boolTop++;
                        }
                    }
                    else {
                        // Handle extended operations
                        (intTop, boolTop, execTop) = handleExtendedOpcodes(
                            op, intStack, boolStack, execStack, intTop, boolTop, execTop
                        );
                    }
                }
                else if (uint8(op) >= OPCODE_BOOL_OFFSET && uint8(op) <= uint8(OpCode.BOOL_RAND)) {
                    // BOOL OPCODES
                    if (op == OpCode.BOOL_DUP) {
                        // dup top
                        if (boolTop > 0) {
                            boolStack[boolTop] = boolStack[boolTop - 1];
                            boolTop++;
                        }
                    }
                    else if (op == OpCode.BOOL_POP) {
                        // pop top
                        if (boolTop > 0) {
                            boolTop--;
                        }
                    }
                    else if (op == OpCode.BOOL_SWAP) {
                        // swap top 2
                        if (boolTop > 1) {
                            (boolStack[boolTop - 1], boolStack[boolTop - 2]) = (boolStack[boolTop - 2], boolStack[boolTop - 1]);
                        }
                    }
                    else if (op == OpCode.BOOL_FLUSH) {
                        // empty stack
                        boolTop = 0;
                    }
                    else if (op == OpCode.BOOL_STACKDEPTH) {
                        // push bool depth onto int stack
                        intStack[intTop] = int256(boolTop);
                        intTop++;
                    }
                    else if (op == OpCode.BOOL_NOT) {
                        // push NOT of top
                        if (boolTop > 0) {
                            boolStack[boolTop - 1] = !boolStack[boolTop - 1];
                        }
                    }
                    else if (op == OpCode.BOOL_AND) {
                        // push AND of top 2
                        if (boolTop > 1) {
                            boolStack[boolTop - 2] = boolStack[boolTop - 1] && boolStack[boolTop - 2];
                            boolTop--;
                        }
                    }
                    else if (op == OpCode.BOOL_OR) {
                        // push OR of top 2
                        if (boolTop > 1) {
                            boolStack[boolTop - 2] = boolStack[boolTop - 1] || boolStack[boolTop - 2];
                            boolTop--;
                        }
                    }
                    else if (op == OpCode.BOOL_EQ) {
                        // push True if top 2 equal
                        if (boolTop > 1) {
                            boolStack[boolTop - 2] = boolStack[boolTop - 1] == boolStack[boolTop - 2];
                            boolTop--;
                        }
                    }
                    else if (op == OpCode.BOOL_FROMFLOAT) {
                        // FLOAT is not implemented yet
                    }
                    else if (op == OpCode.BOOL_FROMINTEGER) {
                        // push True if top int not eq 0
                        if (intTop > 0) {
                            boolStack[boolTop] = intStack[intTop - 1] != int256(0);
                            boolTop++;
                            intTop--;
                        }
                    }
                    else if (op == OpCode.BOOL_ROT) {
                        // rotate [top - 3, top - 2, top - 1] into -> [top - 2, top - 1, top - 3]
                        if (boolTop > 2) {
                            (boolStack[boolTop - 1], boolStack[boolTop - 2], boolStack[boolTop - 3]) = (boolStack[boolTop - 3], boolStack[boolTop - 1], boolStack[boolTop - 2]);
                        }
                    }
                    else if (op == OpCode.BOOL_SHOVE) {
                        // Task: Inserts the top BOOLEAN "deep" in the stack, at the position indexed by the top INTEGER.
                        // Questions: what does "inserts" mean? does it mean that we pop top value and then insert it
                        // at index? does it mean that array size does not change?
                        // what happens with the previous element at this index?
                        // is it correct that this element stays in array?
                        // should it be moved in top or bottom direction from index (index + || -)?
                        // does it mean that we need to reindex the whole stack?
                        // is the simplest solution to reindex to iterate through the whole stack?
                        // do we really want have such loops?
                    }
                    else if (op == OpCode.BOOL_YANK) {
                        // Task: Removes an indexed item from "deep" in the stack and pushes it on top of the stack. The index is taken from the INTEGER stack.
                        // Same question as for SHOVE is valid here.
                        // For example, in our implementation currently we are not really poping
                        // element from an array in POP, we only adjusting the counter.
                        // For removing index we would need to loop over the array to reindex.
                    }
                    else if (op == OpCode.BOOL_YANKDUP) {
                        // Task: Pushes a copy of an indexed item "deep" in the stack onto the top of the stack, without removing the deep item. The index is taken from the INTEGER stack.
                        uint256 index = uint256(intStack[intTop - 1]);
                        if (boolTop > 2) {
                            // pop int stack
                            intTop--;
                            // index in range (reverse array indexes)
                            index++; // adjust as boolTop is length, highest index is boolTop - 1
                            if (index > boolTop) {
                                index = 0;
                            } else {
                                index = boolTop - index;
                            }
                            // push value on bool stack
                            boolStack[boolTop] = boolStack[index];
                            boolTop++;
                        }
                    }
                    else if (op == OpCode.BOOL_DEFINE) {
                        // NAME is not implemented yet
                    }
                    else if (op == OpCode.BOOL_RAND) {
                        // Pushes a random BOOLEAN.
                        uint256 randomNum = uint256(keccak256(abi.encodePacked(block.prevrandao, block.timestamp, msg.sender)));
                        boolStack[boolTop] = extractBool(randomNum);
                        boolTop++;
                    }
                }
            }
            else if (tag == CodeTag.INT_LITERAL) {
                int32 val = extractInt32(topDesc);
                intStack[intTop] = int256(val);
                intTop++;
            }
            else if (tag == CodeTag.BOOL_LITERAL) {
                bool val = extractBool(topDesc);
                boolStack[boolTop] = val;
                boolTop++;
            }
            else if (tag == CodeTag.SUBLIST) {
                // parse sublist => push in reverse
                uint32 off = getOffset(topDesc);
                uint32 len = getLength(topDesc);

                if (off + len <= code.length) {
                    uint256[] memory parsed = parseSublist(code, off, len);
                    for (uint256 p = 0; p < parsed.length; p++) {       // index 0     1     2
                        execStack[execTop] = parsed[parsed.length - 1 - p]; // [0x01, 0x20, 0x0a]
                        execTop++;
                    }
                }
            }
            else {
                // NO_TAG => do nothing
            }
        }

        // E) RETURN
        finalCodeStack = new uint256[](codeTop);
        for (uint256 i = 0; i < codeTop; i++) {
            finalCodeStack[i] = codeStack[i];
        }

        finalExecStack = new uint256[](execTop);
        for (uint256 i = 0; i < execTop; i++) {
            finalExecStack[i] = execStack[i];
        }

        finalIntStack = new int256[](intTop);
        for (uint256 i = 0; i < intTop; i++) {
            finalIntStack[i] = intStack[i];
        }

        finalBoolStack = new bool[](boolTop);
        for (uint256 i = 0; i < boolTop; i++) {
            finalBoolStack[i] = boolStack[i];
        }
    }
}
