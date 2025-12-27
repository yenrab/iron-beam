-module(test_operations).
-export([divide_test/2, compare_test/2, negate_test/1, boolean_test/2]).

divide_test(X, Y) ->
    X / Y.

compare_test(X, Y) ->
    {X < Y, X > Y, X == Y, X /= Y}.

negate_test(X) ->
    -X.

boolean_test(X, Y) ->
    {X and Y, X or Y, not X}.
