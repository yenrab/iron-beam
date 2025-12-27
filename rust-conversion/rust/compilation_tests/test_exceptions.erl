-module(test_exceptions).
-export([try_catch/0, try_after/0]).

%% Try with catch clauses
try_catch() ->
    try
        error(badarg)
    catch
        error:Reason -> {caught_error, Reason};
        throw:Term -> {caught_throw, Term};
        exit:Reason -> {caught_exit, Reason}
    end.

%% Try with after block
try_after() ->
    try
        ok
    after
        io:format("cleanup~n")
    end.
