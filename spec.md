### Specification for how the data will flow

##### User Actions

Each user prompt will pass through a setfit classifier that captures the intent (1-9)

1. Question for item[s] availability
   -> semantic search -> reranker -> list top N
2. Add item (quantity) to cart {check if available}
   -> tendero parser SLM
   -> for each element in the parsed array, check the top 1 result in the /products list
   -> check if that product is available on the inventory with the specified quantity
   -> render list of added items and its quantities, with buttons to confirm for every element, and modifiable quantities. If inventory isn't enough, notify the user and suggest the available amount if there's any
3. Wipe cart
   -> direct operation
4. Delete item from cart
   -> tendero parser SLM with cart context
   -> for every element in the parsed array, render a delete suggestion with an X button, and below all the elements, a button to confirm deletion. If it's only one element, don't render an X button at all
5. Change item in cart quantity
   -> tendero parser SLM with cart context
   -> for every item in the parsed array, do the corresponding operation (change | add | subtract) to the associated element in the cart
6. List cart
   -> direct operation
7. Buy (show ticket)
   -> direct operation
