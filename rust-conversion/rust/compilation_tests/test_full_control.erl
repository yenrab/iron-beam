-module(test_full_control).
-export([case_test/1, if_test/1, combined_test/1]).

case_test(X) ->
    case X of
        1 -> 100;
        2 -> 200;
        _ -> 0
    end.

if_test(X) ->
    if X > 0 ->
        positive;
       X < 0 ->
        negative;
       true ->
        zero
    end.

combined_test(X) ->
    case X of
        Y when Y > 0 ->
            if Y > 10 ->
                big;
               true ->
                small
            end;
        _ ->
            zero
    end.
