-module(test_control_flow).
-export([case_test/1, simple_case/1]).

case_test(X) ->
    case X of
        1 -> 10;
        2 -> 20;
        _ -> 0
    end.

simple_case(X) ->
    case X of
        Y -> Y + 1
    end.
