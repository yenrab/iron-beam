-module(test_funs).
-export([simple_fun/0, higher_order/1, named_fun/0]).

%% Simple anonymous function
simple_fun() ->
    F = fun(X) -> X * 2 end,
    F(5).

%% Higher-order function usage
higher_order(List) ->
    F = fun(X) -> X + 1 end,
    lists:map(F, List).

%% Named fun
named_fun() ->
    F = fun Fact(0) -> 1;
            Fact(N) -> N * Fact(N-1)
        end,
    F(5).
