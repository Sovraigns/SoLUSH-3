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
     *  0x03 => SUBLIST
     *  0x00 0x0B => subLen=11
     *  Then the next 11 bytes are:
     *    0x02 + (00 00 00 0A) => INT_LITERAL(10)
     *    0x02 + (00 00 00 20) => INT_LITERAL(32)
     *    0x01 => INTEGER_PLUS
     *
     *  So total = 1 + 2 + (1+4 + 1+4 + 1) = 14 bytes
     *  => 0x03 00 0B 02 00 00 00 0A 02 00 00 00 20 01
     */
    function test_LiteralsAndPlus() public view {
        // The final code array (14 bytes):
        //  [0] = 0x03
        //  [1..2] = 0x000B
        //  [3] =  0x02
        //  [4..7] = 0x0000000A => 10
        //  [8] =  0x02
        //  [9..12] = 0x00000020 => 32
        //  [13] = 0x01 => plus

        bytes memory code = hex"03000B020000000A020000002001";        

        // Or fully concatenated:
        // 0x03 00 0B 02 00 00 00 0A 02 00 00 00 20 01

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

        // Run it
        (
            ,
            ,
            int256[] memory finalIntStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack);

        // Should produce finalIntStack = [42]
        assertEq(finalIntStack.length, 1, "finalIntStack should have length 1");
        assertEq(finalIntStack[0], 42, "Should be 10+32=42");
    }

    /**
     * @notice Test pushing literal 10, literal 3, then INTEGER_MINUS => (10 - 3) = 7
     *
     * The plan:
     *  0x03 => SUBLIST token
     *  0x00 0x0B => subLen=11
     *
     *  Then 11 bytes:
     *    0x02 + (00 00 00 0A) => INT_LITERAL(10)
     *    0x02 + (00 00 00 03) => INT_LITERAL(3)
     *    0x04 => INTEGER_MINUS (assuming 0x04 is assigned to minus)
     */
    function test_Minus() public view {
        // We'll produce 14 total bytes, same structure as the plus example
        // Format: [0x03, 0x000B, 0x02(10), 0x03(3), 0x04 => minus]

        bytes memory code = hex"03000B020000000A020000000304";
        // Breaking it down:
        // 0x03          => SUBLIST
        // 0x00 0x0B     => length=11
        // 0x02 0000000A => INT_LITERAL(10)
        // 0x02 00000003 => INT_LITERAL(3)
        // 0x04          => INTEGER_MINUS

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

        // Run
        (
            ,
            ,
            int256[] memory finalIntStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack);

        // Check final result => should be [7]
        assertEq(finalIntStack.length, 1, "finalIntStack should have length 1");
        assertEq(finalIntStack[0], 7, "Should be 10 - 3 = 7");
    }

    /**
     * @notice Test pushing literal 10, literal 3, then INTEGER_MULT => (10 * 3) = 30
     *
     * The plan:
     *  0x03 => SUBLIST token
     *  0x00 0x0B => subLen=11
     *
     *  Then 11 bytes:
     *    0x02 + (00 00 00 0A) => INT_LITERAL(10)
     *    0x02 + (00 00 00 03) => INT_LITERAL(3)
     *    0x05 => INTEGER_MULT
     */
    function test_Mult() public view {
        // We'll produce 14 total bytes, same structure as the plus example
        // Format: [0x03, 0x000B, 0x02(10), 0x03(3), 0x05 => mul]

        bytes memory code = hex"03000B020000000A020000000305";
        // Breaking it down:
        // 0x03          => SUBLIST
        // 0x00 0x0B     => length=11
        // 0x02 0000000A => INT_LITERAL(10)
        // 0x02 00000003 => INT_LITERAL(3)
        // 0x05          => INTEGER_MUL

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

        // Run
        (
            ,
            ,
            int256[] memory finalIntStack
        ) = interpreter.runInterpreter(code, initCodeStack, initExecStack, initIntStack);

        // Check final result => should be [30]
        assertEq(finalIntStack.length, 1, "finalIntStack should have length 1");
        assertEq(finalIntStack[0], 30, "Should be 10 * 3 = 30");
    }
}
