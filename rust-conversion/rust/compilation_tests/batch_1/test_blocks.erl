-module(test_blocks).
-export([simple_block/0, complex_block/0]).

%% Simple block expression
simple_block() ->
    begin
        X = 1,
        Y = 2,
        X + Y
    end.

%% Complex block with side effects
complex_block() ->
    begin
        io:format("Starting~n"),
        Result = 42,
        io:format("Result: ~p~n", [Result]),
        Result
    end.
