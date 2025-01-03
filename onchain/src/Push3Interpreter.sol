// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/console.sol";

/**
 * @title Push3Interpreter
 * @dev Demonstrates a minimal Push3-like interpreter that uses
 *      a token-based bytecode approach.
 */
contract Push3Interpreter {
    // -----------------------------------------------------
    // 0. CONSTANTS
    // -----------------------------------------------------
    uint8 internal constant OPCODE_BOOL_OFFSET = 0x6;

    // -----------------------------------------------------
    // 1. ENUMS
    // -----------------------------------------------------
    enum CodeTag {
        NO_TAG,       // 0
        INSTRUCTION,  // 1
        INT_LITERAL,  // 2
        SUBLIST,      // 3
        BOOL_LITERAL  // 4
    }

    enum OpCode {
        NOOP,            // 0
        INTEGER_PLUS,    // 1
        INTEGER_MINUS,   // 2
        INTEGER_MULT,    // 3
        INTEGER_DUP,     // 4
        INTEGER_POP,     // 5
        BOOL_DUP,        // OPCODE_BOOL_OFFSET + 0
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
        BOOL_RAND        // OPCODE_BOOL_OFFSET + 16
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
     * @dev Read 4 bytes from `code` at `start` as uint32.
     */
    function readUint32(bytes calldata code, uint32 start) internal pure returns (uint32 val) {
        require(start + 4 <= code.length, "readUint32 out of range");
        uint256 word;
        assembly {
            let buf := mload(0x40) // free memory
            // copy exactly 4 bytes from code into buf
            calldatacopy(buf, add(code.offset, start), 4)
            word := mload(buf)
        }
        val = uint32(word >> 224);
    }

    function readBool(bytes calldata code, uint32 start) internal pure returns (bool val) {
        require(start + 1 <= code.length, "readBool out of range");
        uint256 word;
        assembly {
            let buf := mload(0x40) // free memory
            // copy exactly 1 byte from code into buf
            calldatacopy(buf, add(code.offset, start), 1)
            // assign 1 byte to a word
            word := shr(248, mload(buf))
        }
        val = word & 1 == 1;
    }

    /**
     * @dev Read 2 bytes from `code` at `start` as uint16.
     */
    function readUint16(bytes calldata code, uint32 start) internal pure returns (uint16 val) {
        require(start + 2 <= code.length, "readUint16 out of range");
        uint256 word;
        assembly {
            let buf := mload(0x40)
            calldatacopy(buf, add(code.offset, start), 2)
            word := mload(buf)
            val := and(word, 0xffff)
        }
        // Not sure this is correct
        val = uint16(word >> 240);
    }

    // -----------------------------------------------------
    // 4. SUBLIST PARSING
    // -----------------------------------------------------
    /**
     * Token format:
     *   0x00 => NOOP
     *   0x01 => INTEGER_PLUS
     *   0x02 => INT_LITERAL => read next 4 bytes => int32
     *   0x03 => SUBLIST => read next 2 bytes => subLen => parse that
     *   0x04 => INTEGER_MINUS
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
                // INTEGER_PLUS
                temp[count] = makeInstruction(OpCode.INTEGER_PLUS);
                count++;
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
            else if (tokenType == 0x04) {
                // INTEGER_PLUS
                temp[count] = makeInstruction(OpCode.INTEGER_MINUS);
                count++;
            }
            else if (tokenType == 0x05) {
                // INTEGER_PLUS
                temp[count] = makeInstruction(OpCode.INTEGER_MULT);
                count++;
            }
            else if (tokenType == 0x06) {
                temp[count] = makeInstruction(OpCode.INTEGER_DUP);
                count++;
            }
            else if (tokenType == 0x07) {
                temp[count] = makeInstruction(OpCode.INTEGER_POP);
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
    // 5. MAIN INTERPRETER
    // -----------------------------------------------------
    function runInterpreter(
        bytes calldata code,
        uint256[] calldata initCodeStack,
        uint256[] calldata initExecStack,
        int256[] calldata initIntStack
    )
        external
        pure
        returns (
            uint256[] memory finalCodeStack,
            uint256[] memory finalExecStack,
            int256[]  memory finalIntStack
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

        // D) MAIN LOOP
        while (execTop > 0) {
            uint256 topDesc = execStack[execTop - 1];
            execTop--;

            CodeTag tag = getTag(topDesc);

            if (tag == CodeTag.INSTRUCTION) {
                OpCode op = getOpCode(topDesc);
                if (op == OpCode.NOOP) {
                    // do nothing
                }
                else if (op == OpCode.INTEGER_PLUS) {
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
            }
            else if (tag == CodeTag.INT_LITERAL) {
                int32 val = extractInt32(topDesc);
                intStack[intTop] = int256(val);
                intTop++;
            }
            else if (tag == CodeTag.SUBLIST) {
                // parse sublist => push in reverse
                uint32 off = getOffset(topDesc);
                uint32 len = getLength(topDesc);

                if (off + len <= code.length) {
                    uint256[] memory parsed = parseSublist(code, off, len);
                    for (uint256 p = 0; p < parsed.length; p++) {
                        execStack[execTop] = parsed[parsed.length - 1 - p];
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
    }
}
