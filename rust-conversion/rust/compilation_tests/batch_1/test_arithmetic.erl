-module(test_arithmetic).
-export([add/2, multiply/2, tuple_test/0, list_test/0]).

add(X, Y) ->
    X + Y.

multiply(X, Y) ->
    X * Y.

tuple_test() ->
    {1, 2, 3}.

list_test() ->
    [1, 2, 3].
