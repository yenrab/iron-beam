-module(test_receive).
-export([simple_receive/0, timeout_receive/0]).

%% Simple receive
simple_receive() ->
    self() ! hello,
    receive
        hello -> ok;
        _ -> error
    end.

%% Receive with timeout
timeout_receive() ->
    receive
        message -> received;
        _ -> timeout
    after 1000 ->
        timeout
    end.
