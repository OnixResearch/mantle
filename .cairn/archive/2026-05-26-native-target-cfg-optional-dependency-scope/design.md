# Design

Native target-cfg dependency planning now separates cfg-table selection from dependency selection.

1. Evaluate the cfg predicate with the bounded native evaluator.
2. For each dependency in a selected cfg table:
   - non-optional dependencies remain required and must resolve through a path or declared registry source;
   - optional dependencies are selected only when native selected feature facts directly name the dependency (`name` or `dep:name`);
   - optional dependencies with no selected feature/source are emitted as `not-selected-optional` facts and do not enter `path_dependencies`.
3. Missing sources still block for required dependencies, so normal topology execution cannot silently claim an undeclared required edge.

This intentionally avoids recursive Cargo feature solving. The slice removes over-claiming of optional target dependencies while keeping Mantle's native receipts explicit and bounded.
