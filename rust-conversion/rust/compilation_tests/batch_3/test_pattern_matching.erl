-module(test_pattern_matching).
-export([factorial/1, fib/1, silly_length/1, head/1, tuple_match/1, list_match/1, guard_test/1, multiple_clauses/1]).

%% Factorial using pattern matching
factorial(0) -> 1;
factorial(N) when N > 0 -> N * factorial(N - 1).

%% Fibonacci using pattern matching
fib(0) -> 0;
fib(1) -> 1;
fib(N) when N > 1 -> fib(N - 1) + fib(N - 2).

%% Length of list using pattern matching
silly_length([]) -> 0;
silly_length([_ | Tail]) -> 1 + length(Tail).

%% Head of list using pattern matching
head([H | _]) -> H.

%% Tuple pattern matching
tuple_match({ok, Value}) -> Value;
tuple_match({error, Reason}) -> {error, Reason};
tuple_match(_) -> unknown.

%% List pattern matching
list_match([]) -> empty;
list_match([X]) -> {single, X};
list_match([X, Y | Rest]) -> {multiple, X, Y, Rest}.

%% Guard test with pattern matching
guard_test(X) when is_integer(X), X > 0 -> positive;
guard_test(X) when is_integer(X), X < 0 -> negative;
guard_test(0) -> zero;
guard_test(_) -> not_integer.

%% Multiple clauses with different patterns
multiple_clauses({tag, Value}) -> {tagged, Value};
multiple_clauses([H | T]) -> {list, H, T};
multiple_clauses(X) when is_atom(X) -> {atom, X};
multiple_clauses(X) -> {other, X}.
