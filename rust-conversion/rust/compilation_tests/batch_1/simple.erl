-module(simple).
-export([test/0, identity/1]).

test() -> 42.

identity(X) -> X.
