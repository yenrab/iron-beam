-module(test_multi).
-export([inc/1, dec/1, add/2]).

inc(X) ->
    X + 1.

dec(X) ->
    X - 1.

add(X, Y) ->
    X + Y.