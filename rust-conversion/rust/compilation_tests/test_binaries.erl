-module(test_binaries).
-export([simple_binary/0, sized_binary/0, binary_match/1]).

%% Simple binary construction
simple_binary() ->
    <<1, 2, 3, 4>>.

%% Binary with size specifications
sized_binary() ->
    <<42:8, 123:16, "hello">>.

%% Binary pattern matching (would need case expression support)
binary_match(Binary) ->
    case Binary of
        <<Size:8, Data:Size/binary, Rest/binary>> -> {Size, Data, Rest};
        _ -> unmatched
    end.
