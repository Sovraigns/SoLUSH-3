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

        bytes memory code = hex"040009020000000A0803010A";

        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,
            12,
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
            int256[] memory finalIntStack,
            bool[] memory finalBoolStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        assertEq(finalIntStack.length, 2, "finalIntStack should have length 1");
        assertEq(finalIntStack[0], finalIntStack[1], "The top 2 values on the int stack should be equal.");
        assertEq(finalBoolStack.length, 2, "finalBoolStack should have length 2");
        assertEq(finalBoolStack[0], finalBoolStack[1], "The top 2 values on the bool stack should be equal.");
    }

    function test_Pop() public view {

        bytes memory code = hex"040009020000000A0903010B";

        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,
            12,
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
            int256[] memory finalIntStack,
            bool[] memory finalBoolStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        assertEq(finalIntStack.length, 0, "finalIntStack should be empty");
        assertEq(finalBoolStack.length, 0, "finalBoolStack should be empty");
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

    // BOOL STACK

    /**
     * @notice Test pushing literal True, literal False, then BOOL_SWAP => [False, True]
     *
     * The plan:
     *  0x04 => SUBLIST token
     *  0x00 0x05 => subLen=5
     *
     *  Then 5 bytes:
     *    0x03 + (01) => BOOL_LITERAL(True)
     *    0x03 + (00) => BOOL_LITERAL(False)
     *    0x0C => BOOL_SWAP
     */
    function test_BoolSwap() public view {
        // We'll produce 8 total bytes, same structure
        // Format: [0x04, 0x0005, 0x03(01), 0x03(00), 0x0C => bool_swap]

        bytes memory code = hex"040005030103000C";
        // Breaking it down:
        // 0x04          => SUBLIST
        // 0x00 0x05     => length=5
        // 0x03 0x01     => BOOL_LITERAL(True)
        // 0x03 0x00     => BOOL_LITERAL(False)
        // 0x0C          => BOOL_SWAP

        // We'll parse offset=0, length=8
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,   // offset
            8,  // length
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
            ,
            bool[] memory finalBoolStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Check final result => should be [True, False] swapped into [False, True]
        assertEq(finalBoolStack.length, 2, "finalBoolStack should have length 2");
        assertEq(finalBoolStack[0], false, "Should be False");
        assertEq(finalBoolStack[1], true, "Should be True");
    }

    /**
     * @notice Test pushing literal True, literal False, then BOOL_FLUSH => []
     *         Result should be empty stack.
     */
    function test_BoolFlush() public view {
        // We'll produce 8 total bytes, same structure
        // Format: [0x04, 0x0005, 0x03(01), 0x03(00), 0x0D => bool_flush]

        bytes memory code = hex"040005030103000D";
        // Breaking it down:
        // 0x04          => SUBLIST
        // 0x00 0x05     => length=5
        // 0x03 0x01     => BOOL_LITERAL(True)
        // 0x03 0x00     => BOOL_LITERAL(False)
        // 0x0D          => BOOL_FLUSH

        // We'll parse offset=0, length=8
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,   // offset
            8,  // length
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
            ,
            bool[] memory finalBoolStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Check final result => should be empty bool stack
        assertEq(finalBoolStack.length, 0, "finalBoolStack should have length 0");
    }

    /**
     * @notice Test pushing literal True, literal False, then BOOL_STACKDEPTH => intStack[2]
     *         Result should be number of pushed literals as intStack top.
     */
    function test_BoolStackDepth() public view {
        // We'll produce 8 total bytes, same structure
        // Format: [0x04, 0x0005, 0x03(01), 0x03(00), 0x0E => bool_stackdepth]

        bytes memory code = hex"040005030103000E";
        // Breaking it down:
        // 0x04          => SUBLIST
        // 0x00 0x05     => length=5
        // 0x03 0x01     => BOOL_LITERAL(True)
        // 0x03 0x00     => BOOL_LITERAL(False)
        // 0x0E          => BOOL_STACKDEPTH

        // We'll parse offset=0, length=8
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,   // offset
            8,  // length
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
            int256[] memory finalIntStack,
            bool[] memory finalBoolStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Check final result => should be empty bool stack
        assertEq(finalIntStack.length, 1, "finalIntStack should have length 1");
        assertEq(finalBoolStack.length, 2, "finalBoolStack should have length 2");
        assertEq(finalIntStack[0], 2, "IntStack top value should be 2, as we pushed 2 literals onto bool stack");
    }

    /**
     * @notice Test pushing literal True, then BOOL_NOT => boolStack[False]
     *         Result should be False on top of bool stack.
     */
    function test_BoolNot() public view {
        // We'll produce 6 total bytes, same structure
        // Format: [0x04, 0x0003, 0x03(01), 0x0F => bool_not]

        bytes memory code = hex"04000303010F";
        // Breaking it down:
        // 0x04          => SUBLIST
        // 0x00 0x03     => length=3
        // 0x03 0x01     => BOOL_LITERAL(True)
        // 0x0F          => BOOL_NOT

        // We'll parse offset=0, length=6
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,   // offset
            6,  // length
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
            ,
            bool[] memory finalBoolStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Check final result => should be False on top of bool stack
        assertEq(finalBoolStack.length, 1, "finalBoolStack should have length 1");
        assertEq(finalBoolStack[0], false, "Expected to be False after logical NOT on True");
    }

    /**
     * @notice Test pushing literal True, literal True, then BOOL_AND => boolStack[True]
     *         Result should be True on top of bool stack.
     */
    function test_BoolAnd() public view {
        // We'll produce 8 total bytes, same structure
        // Format: [0x04, 0x0005, 0x03(01), 0x03(01) 0x10 => bool_and]

        bytes memory code = hex"0400050301030110";
        // Breaking it down:
        // 0x04          => SUBLIST
        // 0x00 0x05     => length=5
        // 0x03 0x01     => BOOL_LITERAL(True)
        // 0x03 0x01     => BOOL_LITERAL(True)
        // 0x10          => BOOL_AND

        // We'll parse offset=0, length=8
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,   // offset
            8,  // length
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
            ,
            bool[] memory finalBoolStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Check final result => should be True on top of bool stack
        assertEq(finalBoolStack.length, 1, "finalBoolStack should have length 1");
        assertEq(finalBoolStack[0], true, "Expected to be True after logical AND on True and True");
    }

    /**
     * @notice Test pushing literal True, literal False, then BOOL_OR => boolStack[True]
     *         Result should be True on top of bool stack.
     */
    function test_BoolOr() public view {
        // We'll produce 8 total bytes, same structure
        // Format: [0x04, 0x0005, 0x03(01), 0x03(00) 0x11 => bool_or]

        bytes memory code = hex"0400050301030011";
        // Breaking it down:
        // 0x04          => SUBLIST
        // 0x00 0x05     => length=5
        // 0x03 0x01     => BOOL_LITERAL(True)
        // 0x03 0x00     => BOOL_LITERAL(False)
        // 0x11          => BOOL_OR

        // We'll parse offset=0, length=8
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,   // offset
            8,  // length
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
            ,
            bool[] memory finalBoolStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Check final result => should be True on top of bool stack
        assertEq(finalBoolStack.length, 1, "finalBoolStack should have length 1");
        assertEq(finalBoolStack[0], true, "Expected to be True after logical OR on True and False");
    }

    /**
     * @notice Test pushing literal True, literal False, then BOOL_EQ => boolStack[False]
     *         Result should be False on top of bool stack.
     */
    function test_BoolEq() public view {
        // We'll produce 8 total bytes, same structure
        // Format: [0x04, 0x0005, 0x03(01), 0x03(00) 0x12 => bool_eq]

        bytes memory code = hex"0400050301030012";
        // Breaking it down:
        // 0x04          => SUBLIST
        // 0x00 0x05     => length=5
        // 0x03 0x01     => BOOL_LITERAL(True)
        // 0x03 0x00     => BOOL_LITERAL(False)
        // 0x12          => BOOL_EQ

        // We'll parse offset=0, length=8
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,   // offset
            8,  // length
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
            ,
            bool[] memory finalBoolStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Check final result => should be False on top of bool stack
        assertEq(finalBoolStack.length, 1, "finalBoolStack should have length 1");
        assertEq(finalBoolStack[0], false, "Expected to be False as True is not equal False");
    }

    /**
     * @notice Test pushing literal 0x2145, then BOOL_FROMINTEGER => boolStack[True]
     *         Result should be True on top of bool stack.
     */
    function test_BoolFromInteger() public view {
        // We'll produce 9 total bytes, same structure
        // Format: [0x04, 0x0004, 0x02(0x00002145), 0x14 => bool_frominteger]

        bytes memory code = hex"040006020000214514";
        // Breaking it down:
        // 0x04            => SUBLIST
        // 0x00 0x04       => length=6
        // 0x02 0x00002145 => INT_LITERAL(0x2145)
        // 0x14            => BOOL_FROMINTEGER

        // We'll parse offset=0, length=9
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,   // offset
            9,  // length
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
            int256[] memory finalIntStack,
            bool[] memory finalBoolStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Check final result => should be True on bool stack
        assertEq(finalIntStack.length, 0, "finalIntStack should have length 0");
        assertEq(finalBoolStack.length, 1, "finalBoolStack should have length 1");
        assertEq(finalBoolStack[0], true, "boolStack top value should be True, as we pushed non-zero literal onto int stack");
    }

    /**
     * @notice Test pushing literal True, literal False, literal True, then BOOL_ROT => boolStack[False, True, True]
     *         Result should be rotated bool stack from [True, False, True] into [False, True, True].
     */
    function test_BoolRot() public view {
        // We'll produce 10 total bytes, same structure
        // Format: [0x04, 0x0007, 0x03(01), 0x03(00), 0x03(01), 0x15 => bool_rot]

        bytes memory code = hex"04000703010300030115";
        // Breaking it down:
        // 0x04          => SUBLIST
        // 0x00 0x07     => length=7
        // 0x03 0x01     => BOOL_LITERAL(True)
        // 0x03 0x00     => BOOL_LITERAL(False)
        // 0x03 0x01     => BOOL_LITERAL(True)
        // 0x15          => BOOL_ROT

        // We'll parse offset=0, length=10
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,   // offset
            10,  // length
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
            ,
            bool[] memory finalBoolStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Check final result => should be [False, True, True] bool stack
        assertEq(finalBoolStack.length, 3, "finalBoolStack should have length 3");
        assertEq(finalBoolStack[0], false, "Expected to be False as value from middle position moved to bottom of the stack");
        assertEq(finalBoolStack[1], true, "Expected to be True as value from top position moved to middle of the stack");
        assertEq(finalBoolStack[2], true, "Expected to be True as value from bottom position moved to top of the stack");
    }

    /**
     * @notice Test pushing literal 0x02, literal True, literal False, literal True, then BOOL_YANKDUP => boolStack[True, False, True, True]
     *         Result should be copied True on top of bool stack [True, False, True, True]
     */
    function test_BoolYankDup() public view {
        // We'll produce 15 total bytes, same structure
        // Format: [0x04, 0x000C, 0x02(00 00 00 02) 0x03(01), 0x03(00), 0x03(01), 0x18 => bool_yankdup]

        bytes memory code = hex"04000C020000000203010300030118";
        // Breaking it down:
        // 0x04          => SUBLIST
        // 0x00 0x0C     => length=12
        // 0x02 0x00000002 => INT_LITERAL(0x02)
        // 0x03 0x01     => BOOL_LITERAL(True)
        // 0x03 0x00     => BOOL_LITERAL(False)
        // 0x03 0x01     => BOOL_LITERAL(True)
        // 0x18          => BOOL_YANKDUP

        // We'll parse offset=0, length=15
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,   // offset
            15,  // length
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
            ,
            bool[] memory finalBoolStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Check final result => should be [True, False, True, True] bool stack
        assertEq(finalBoolStack.length, 4, "finalBoolStack should have length 4");
        assertEq(finalBoolStack[0], true, "Expected to be True as first value from bool literal");
        assertEq(finalBoolStack[1], false, "Expected to be False as second value from bool literal");
        assertEq(finalBoolStack[2], true, "Expected to be True as third value from bool literal");
        assertEq(finalBoolStack[3], true, "Expected to be True as copied value at index 2, which is index 0 for an array");
    }

    /**
     * @notice Test pushing BOOL_RAND => boolStack[True] or boolStack[False]
     *         Result should be either False or True on top of bool stack.
     */
    function test_BoolRand() public {
        // We'll produce 4 total bytes, same structure
        // Format: [0x04, 0x0001, 0x1A => bool_rand]

        bytes memory code = hex"0400011A";
        // Breaking it down:
        // 0x04          => SUBLIST
        // 0x00 0x01     => length=1
        // 0x1A          => BOOL_RAND

        // We'll parse offset=0, length=4
        uint256 sublistDesc = interpreter.makeDescriptor(
            Push3Interpreter.CodeTag.SUBLIST,
            0,   // offset
            4,  // length
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
            ,
            bool[] memory finalBoolStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);

        // Check final result => should be random bool on top of bool stack
        assertEq(finalBoolStack.length, 1, "finalBoolStack should have length 1");
        assertEq(finalBoolStack[0], false, "Expected to be False for default prevrandao, timestamp and msg.sender");
        // move to another timestamp to get different random result
        vm.warp(1736055577);
        (
            ,
            ,
            ,
            finalBoolStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack, initBoolStack);
        assertEq(finalBoolStack.length, 1, "finalBoolStack should have length 1");
        assertEq(finalBoolStack[0], true, "Expected to be True for configured timestamp and default prevrandao, msg.sender");
    }
}
