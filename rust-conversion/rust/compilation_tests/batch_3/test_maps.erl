-module(test_maps).
-export([create_map/0, update_map/1, access_map/2]).

%% Create a new map
create_map() ->
    #{name => "John", age => 25, city => "NYC"}.

%% Update an existing map
update_map(Map) ->
    Map#{age => 26, country => "USA"}.

%% Access map values (this would need pattern matching)
access_map(Key, Map) ->
    case Map of
        #{Key := Value} -> Value;
        _ -> undefined
    end.
