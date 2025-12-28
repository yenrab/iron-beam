-module(test_comprehensions).
-export([simple_list_comp/1, filtered_list_comp/1, nested_list_comp/1, generator_list_comp/2]).

%% Simple list comprehension - double each element
simple_list_comp(List) ->
    [X * 2 || X <- List].

%% Filtered list comprehension - double only even numbers
filtered_list_comp(List) ->
    [X * 2 || X <- List, X rem 2 == 0].

%% Nested comprehension
nested_list_comp(List) ->
    [[X, Y] || X <- List, Y <- List, X < Y].

%% Multiple generators
generator_list_comp(List1, List2) ->
    [X + Y || X <- List1, Y <- List2].
