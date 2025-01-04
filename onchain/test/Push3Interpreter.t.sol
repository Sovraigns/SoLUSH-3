// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import "forge-std/console.sol";
import {Push3Interpreter} from "../src/Push3Interpreter.sol";

/**
 * @title Push3InterpreterTest
 * @dev Foundry test contract for the updated Push3Interpreter
 */
contract Push3InterpreterTest is Test {
    Push3Interpreter public interpreter;

    function setUp() public {
        interpreter = new Push3Interpreter();
    }

    /**
     * @notice Test a small program that:
     *   1) Pushes literal 10 (int32)
     *   2) Pushes literal 32 (int32)
     *   3) Calls INTEGER_PLUS
     *
     *  We'll store everything in a single SUBLIST:
     *  0x04 => SUBLIST
     *  0x00 0x0B => subLen=11
     *  Then the next 11 bytes are:
     *    0x02 + (00 00 00 0A) => INT_LITERAL(10)
     *    0x02 + (00 00 00 20) => INT_LITERAL(32)
     *    0x05 => INTEGER_PLUS
     *
     *  So total = 1 + 2 + (1+4 + 1+4 + 1) = 14 bytes
     *  => 0x04 00 0B 02 00 00 00 0A 02 00 00 00 20 05
     */
    function test_LiteralsAndPlus() public view {
        // The final code array (14 bytes):
        //  [0] = 0x04
        //  [1..2] = 0x000B
        //  [3] =  0x02
        //  [4..7] = 0x0000000A => 10
        //  [8] =  0x02
        //  [9..12] = 0x00000020 => 32
        //  [13] = 0x05 => plus

        bytes memory code = hex"04000B020000000A020000002005";

        // Or fully concatenated:
        // 0x04 00 0B 02 00 00 00 0A 02 00 00 00 20 05

        // We'll parse offset=0, length=14
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,   // offset
            14,  // length
            0
        );

        uint256[] memory initCodeStack = new uint256[](0);
        uint256[] memory initExecStack = new uint256[](1);
        initExecStack[0] = sublistDesc;

        int256[] memory initIntStack = new int256[](0);
        bool[] memory initBoolStack = new bool[](0);

        // Run it
        (
            ,
            ,
            int256[] memory finalIntStack
            ,
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Should produce finalIntStack = [42]
        assertEq(finalIntStack.length, 1, "finalIntStack should have length 1");
        assertEq(finalIntStack[0], 42, "Should be 10+32=42");
    }

    /**
     * @notice Test pushing literal 10, literal 3, then INTEGER_MINUS => (10 - 3) = 7
     *
     * The plan:
     *  0x04 => SUBLIST token
     *  0x00 0x0B => subLen=11
     *
     *  Then 11 bytes:
     *    0x02 + (00 00 00 0A) => INT_LITERAL(10)
     *    0x02 + (00 00 00 03) => INT_LITERAL(3)
     *    0x06 => INTEGER_MINUS (assuming 0x06 is assigned to minus)
     */
    function test_Minus() public view {
        // We'll produce 14 total bytes, same structure as the plus example
        // Format: [0x04, 0x000B, 0x02(10), 0x03(3), 0x06 => minus]

        bytes memory code = hex"04000B020000000A020000000306";
        // Breaking it down:
        // 0x04          => SUBLIST
        // 0x00 0x0B     => length=11
        // 0x02 0000000A => INT_LITERAL(10)
        // 0x02 00000003 => INT_LITERAL(3)
        // 0x06          => INTEGER_MINUS

        // We'll parse offset=0, length=14
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,   // offset
            14,  // length
            0
        );

        uint256[] memory initCodeStack = new uint256[](0);
        uint256[] memory initExecStack = new uint256[](1);
        initExecStack[0] = sublistDesc;

        int256[] memory initIntStack = new int256[](0);
        bool[] memory initBoolStack = new bool[](0);

        // Run
        (
            ,
            ,
            int256[] memory finalIntStack
            ,
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Check final result => should be [7]
        assertEq(finalIntStack.length, 1, "finalIntStack should have length 1");
        assertEq(finalIntStack[0], 7, "Should be 10 - 3 = 7");
    }

    /**
     * @notice Test pushing literal 10, literal 3, then INTEGER_MULT => (10 * 3) = 30
     *
     * The plan:
     *  0x04 => SUBLIST token
     *  0x00 0x0B => subLen=11
     *
     *  Then 11 bytes:
     *    0x02 + (00 00 00 0A) => INT_LITERAL(10)
     *    0x02 + (00 00 00 03) => INT_LITERAL(3)
     *    0x07 => INTEGER_MULT
     */
    function test_Mult() public view {
        // We'll produce 14 total bytes, same structure as the plus example
        // Format: [0x04, 0x000B, 0x02(10), 0x02(3), 0x07 => mul]

        bytes memory code = hex"04000B020000000A020000000307";
        // Breaking it down:
        // 0x04          => SUBLIST
        // 0x00 0x0B     => length=11
        // 0x02 0000000A => INT_LITERAL(10)
        // 0x02 00000003 => INT_LITERAL(3)
        // 0x07          => INTEGER_MUL

        // We'll parse offset=0, length=14
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,   // offset
            14,  // length
            0
        );

        uint256[] memory initCodeStack = new uint256[](0);
        uint256[] memory initExecStack = new uint256[](1);
        initExecStack[0] = sublistDesc;

        int256[] memory initIntStack = new int256[](0);
        bool[] memory initBoolStack = new bool[](0);

        // Run
        (
            ,
            ,
            int256[] memory finalIntStack
            ,
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Check final result => should be [30]
        assertEq(finalIntStack.length, 1, "finalIntStack should have length 1");
        assertEq(finalIntStack[0], 30, "Should be 10 * 3 = 30");
    }

    function test_Dup() public view {

        bytes memory code = hex"040006020000000A08";

        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,
            9,
            0
        );

        uint256[] memory initCodeStack = new uint256[](0);
        uint256[] memory initExecStack = new uint256[](1);
        initExecStack[0] = sublistDesc;

        int256[] memory initIntStack = new int256[](0);
        bool[] memory initBoolStack = new bool[](0);

        // Run
        (
            ,
            ,
            int256[] memory finalIntStack
            ,
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        assertEq(finalIntStack.length, 2, "finalIntStack should have length 1");
        assertEq(finalIntStack[0], finalIntStack[1], "The top 2 values on the stack should be equal.");
    }

    function test_Pop() public view {

        bytes memory code = hex"040006020000000A09";

        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,
            9,
            0
        );

        uint256[] memory initCodeStack = new uint256[](0);
        uint256[] memory initExecStack = new uint256[](1);
        initExecStack[0] = sublistDesc;

        int256[] memory initIntStack = new int256[](0);
        bool[] memory initBoolStack = new bool[](0);

        // Run
        (
            ,
            ,
            int256[] memory finalIntStack
            ,
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        assertEq(finalIntStack.length, 0, "finalIntStack should be empty");
    }

    /**
     * @notice Test with an unknown token (0xF0).
     * We expect the parser to treat it as "unknown => NOOP"
     * and produce a NOOP instruction descriptor.
     * We'll confirm finalIntStack remains empty (or unchanged).
     */
    function test_UnknownToken() public view {
        // We'll create a 5-byte code:
        //  0x04 => SUBLIST
        //  0x00 0x01 => length=1
        //  0xF0 => unknown token
        bytes memory code = hex"040001F0";

        // We'll parse offset=0, length=4
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,   // offset
            4,   // length
            0
        );

        // init stacks
        uint256[] memory initCodeStack = new uint256[](0);
        uint256[] memory initExecStack = new uint256[](1);
        initExecStack[0] = sublistDesc;
        int256[] memory initIntStack = new int256[](0);
        bool[] memory initBoolStack = new bool[](0);

        (
            ,
            ,
            int256[] memory finalIntStack
            ,
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // finalIntStack should remain empty => no literal was pushed
        assertEq(finalIntStack.length, 0, "Expected finalIntStack to remain empty with unknown token => NOOP");
    }

    /**
     * @notice Test a partial literal: we claim to have 4 bytes, but the code array ends early.
     * This should trigger the `else { break; }` branch in parseSublist
     * when trying to read 4 bytes for the INT_LITERAL.
     */
    function test_OutOfRangeLiteral() public view {
        //  0x04 => SUBLIST
        //  0x00 0x04 => length=4
        //  0x02 => INT_LITERAL token
        // Then we have only 1 byte left instead of 4 => insufficient
        //
        // total = 1 + 2 + 1 + 1 = 5 bytes
        // The parser tries to read 4 bytes after seeing 0x04, fails, hits break.
        bytes memory code = hex"04000402FF";

        // parse offset=0, length=5
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,
            5,
            0
        );

        uint256[] memory initCodeStack = new uint256[](0);
        uint256[] memory initExecStack = new uint256[](1);
        initExecStack[0] = sublistDesc;
        int256[] memory initIntStack = new int256[](0);
        bool[] memory initBoolStack = new bool[](0);

        (
            ,
            ,
            int256[] memory finalIntStack
            ,
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Because the parse breaks early, we expect finalIntStack is empty or unchanged
        assertEq(finalIntStack.length, 0, "Expected parse to break on out-of-range literal => no ints pushed");
    }

    /**
     * @notice Test "not enough int args" scenario:
     * We'll push only 1 literal, then do INTEGER_PLUS which needs 2 integers on the stack.
     * In the main loop, it sees intTop < 2 => it does nothing.
     */
    function test_NotEnoughIntArgs() public view {
        bytes memory code = hex"040006020000000307";
        // Breaking it down:
        // 0x04          => SUBLIST
        // 0x00 0x06     => length=6
        // 0x02 00000003 => INT_LITERAL(3)
        // 0x07          => INTEGER_MUL

        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,
            9,
            0
        );

        // set up stacks
        uint256[] memory initCodeStack = new uint256[](0);
        uint256[] memory initExecStack = new uint256[](1);
        initExecStack[0] = sublistDesc;
        int256[] memory initIntStack = new int256[](0);
        bool[] memory initBoolStack = new bool[](0);

        (
            ,
            ,
            int256[] memory finalIntStack
            ,
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // We expect finalIntStack => [5],
        // because the PLUS instruction sees only 1 int => does nothing => 5 remains
        assertEq(finalIntStack.length, 1, "Should have 1 int left");
        assertEq(finalIntStack[0], 3, "Expected stack top=3 after insufficient-args plus");
    }

    /**
     * @notice Test sublist length overflow:
     * subLen claims bigger than the code array. Should trigger else { break; }
     * in the sublist parsing for that sublist.
     */
    function test_SublistLengthOverflow() public view {

        bytes memory code = hex"050010FFFFFF";

        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,
            6,
            0
        );

        // init
        uint256[] memory initCodeStack = new uint256[](0);
        uint256[] memory initExecStack = new uint256[](1);
        initExecStack[0] = sublistDesc;
        int256[] memory initIntStack = new int256[](0);
        bool[] memory initBoolStack = new bool[](0);

        (
            ,
            ,
            int256[] memory finalIntStack
            ,
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Expect no items pushed => finalIntStack empty
        assertEq(finalIntStack.length, 0, "Expected empty stack if sublist length is out of range => parse break");
    }
}
