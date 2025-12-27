-module(test_function_calls).
-export([test/0, add/2, factorial/1]).

test() ->
    add(5, 3).

add(X, Y) ->
    X + Y.

factorial(0) ->
    1;
factorial(N) ->
    N * factorial(N - 1).
