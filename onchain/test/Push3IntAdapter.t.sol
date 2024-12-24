// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {Push3Interpreter} from "../src/Push3Interpreter.sol";
import {Push3IntAdapter} from "../src/Push3IntAdapter.sol";

/**
 * @title Push3IntAdapterTest
 * @dev Foundry test for the Push3IntAdapter contract, 
 *      which composes with the Push3Interpreter.
 */
contract Push3IntAdapterTest is Test {
    Push3Interpreter public interpreter;
    Push3IntAdapter public adapter;

    function setUp() public {
        // 1) Deploy the interpreter
        interpreter = new Push3Interpreter();

        // 2) Deploy the adapter, referencing the interpreter address
        adapter = new Push3IntAdapter(address(interpreter));
    }

    /**
     * @notice Test run1In1Out with a minimal code array. 
     * For example, we might do a program that just leaves its input unchanged 
     * (like a NOOP). 
     */
    function test_run1In1Out_noop() public view {
        bytes memory code = hex"03000100";

        int256 inputVal = 123;
        int256 result = adapter.run1In1Out(code, inputVal);

        // We expect the code does nothing => output == input
        assertEq(result, inputVal, "Expected same value out");
    }

    /**
     * @notice Test run2In1Out. We'll push 5, 7, do an op that leaves 12.
     */
    function test_run2In1Out_plus() public view {
        bytes memory code = hex"03000101";

        int256 in1 = 5;
        int256 in2 = 7;
        int256 result = adapter.run2In1Out(code, in1, in2);
        
        assertEq(result, 12, "Expected 5 + 7 = 12");
    }

    /**
     * @notice Test run1In2Out. Push a constant to a stack with 1.
     * For now, we do a placeholder code. 
     */
    function test_run1In2Out() public view {
        bytes memory code = hex"0300050200000003";

        int256 inVal = 42;
        (int256 top, int256 secondTop) = adapter.run1In2Out(code, inVal);

        assertEq(top, 3, "top stack val mismatch");
        assertEq(secondTop, 42, "second top stack val mismatch");
    }

    /**
     * @notice Test run0InNOut. 
     * We'll feed a code that pushes some constants, and see the final stack.
     */
    function test_run0InNOut() public view {
        
        bytes memory code = hex"03000A02000000030200000005";

        int256[] memory stackResult = adapter.run0InNOut(code);

        assertEq(stackResult.length, 2, "Expected 2 final stack items");
        assertEq(stackResult[0], 3, "Item 0 mismatch");
        assertEq(stackResult[1], 5, "Item 1 mismatch");
    }
}
