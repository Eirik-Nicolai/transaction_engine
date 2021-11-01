# transaction_engine
A toy transaction engine that deals with deposits/withdrawals, as well as disputes/resolves/chargebacks.


### Assumptions made

* If account is locked, money movement is strictly prohibited. 
  * _Disputes can still be put in, as a client would mark a transaction which would cause a dispute on the locked account. There can't be a resolve or chargeback however before the account is unlocked_

* Depostits and withdrawals can't be negative

* A transaction can be disputed/resolved multiple times, but charged back only once

* A record in csv will always have 4 fields, even disputes/resolves/chargebacks

* Disputes will only work for deposits.
  * _Due to how the assignment is written, this is assumed. The idea is that a client mark a withdrawal from their account, which would cause a dispute on the deposit of whatever client account that would have gone to._

* Payments are assumed to all be 100%
  * _This extends on the above assumption. This makes us infer that on an account every deposit has a withdrawal on another account
(even if we don't get it as an input), as aposed to a cash deposit/withdrawal which would be from outside._

* CSV input file is comma-delimited with no whitespace in headers or data
 * Given any whitespace in a record, the record will be ignored


### Design choices

* Used floats despite a bad precision with mathematical operators adue to time constraints and an unfamiliar CSV/Serde package. 
  * Given more time I'd have implemented the Visitor trait to convert it to a u128 as a base data type and convert that back to a float for outputs.

* Using an unordered dataset (hashmap) for speed of finding value to key as we don't care about the order after we store and print

* A threaded design was not implemented as I felt it didn't make sense in the assignment text given with the time constraints. 
  * The inputs were chronological, which means using threads to process input records concurrently could lead to situations where E.G. a dispute is processed in a thread when it hasn't done processing in another, leading to the dispute being ignored even if it is valid. 
  * Other uses of threads would be to have each client have it's own thread to avoid the above issue. 
  * One could also have a thread loading all input while the main thread processes them concurrently. This might be useful if input were given in bulk.

* As specified it's important to think of a larger prod setting with multiple TCP connections sending csv's. 
  * The application and logic it uses only makes sense to me if it's done chronologically _(f. ex. disputes always comes after a transaction)_ so I imagine one would need to implement more checking around timestamps.
  * Multi-threading/async design would be much more useful in this scenario as one would need to deal with a large amount of concurrent streams of information that's to be processed. The application could f. ex. use thread to handle the many TCP connections and asynchrounously process them.

### Other

* Along the application are some of the tests I used during development. Not added are the test files used as the assignment specified **"This test file or any derivative must not be committed".**
