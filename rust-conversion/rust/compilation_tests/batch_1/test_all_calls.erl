-module(test_all_calls).
-export([local_call/1, external_call/1, bif_call/0]).

local_call(X) ->
    double(X).

double(Y) ->
    Y * 2.

external_call(X) ->
    math:sqrt(X).

bif_call() ->
    erlang:self().
