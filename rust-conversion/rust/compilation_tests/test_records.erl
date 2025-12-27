-module(test_records).
-export([create_record/0, access_field/1, update_record/1]).

%% Define a record (this would normally be in a header file)
-record(person, {name, age, city}).

%% Create a record
create_record() ->
    #person{name = "John", age = 25, city = "NYC"}.

%% Access a field
access_field(Person) ->
    Person#person.name.

%% Update a record
update_record(Person) ->
    Person#person{age = Person#person.age + 1}.
