-module(test_advanced_patterns).
-export([record_pattern/1, map_pattern/1, binary_pattern/1]).

%% Record pattern matching (requires record definitions)
-record(person, {name, age}).

record_pattern(Data) ->
    case Data of
        #person{name=Name, age=Age} when Age > 18 -> {adult, Name};
        #person{name=Name} -> {minor, Name};
        _ -> unknown
    end.

%% Map pattern matching
map_pattern(Data) ->
    case Data of
        #{name := Name, age := Age} when Age > 18 -> {adult, Name};
        #{name := Name} -> {person, Name};
        _ -> unknown
    end.

%% Binary pattern matching
binary_pattern(Data) ->
    case Data of
        <<Size:8, Payload:Size/binary, Rest/binary>> -> {Size, Payload, Rest};
        <<1:1, 0:1, Value:6>> -> {bit_field, Value};
        _ -> no_match
    end.
