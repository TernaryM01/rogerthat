![Wordle Solver](https://user-images.githubusercontent.com/51146895/229331944-10d7fc59-262a-4b6e-b9c4-2442fab565d8.gif)

# Background

This Wordle solver started out as a re-do of [roget by Jon Gjengset](https://github.com/jonhoo/roget), faithfully following his live-coded [video on Youtube](https://youtu.be/doFowk4xj7Q) (hence the name, "roger that"). However, along the way, his implementation goes wrong and it becomes worse and worse when he tries to optimize it, that in the end the computation of the optimal next guess doesn't make sense at all. So, this reimplementation diverges from it, and aspires to be a faithful implementation of the ideas put forward in [3blue1brown's viral video](https://www.youtube.com/watch?v=v68zYyaEmEA) on how to make optimal guesses in Wordle by applying information theory. It retains the spirit of trying to create a Wordle solver that is optimal (both in terms of blazing fast computation and making as few guesses as possible), while it also ensures that every optimization trick is mathematically sound.

The first optimization trick is decoupling two levels of for loops into a sequence of two for loops which is missed by Jon (at least throughout his live-coding), which means that this program achieves ***exponential speedup against roget***.

However, roget still looks pretty quick, but it is mainly because it only plays in *hard mode*: every guess is forced to be consistent with all the patterns received so far. (Jon evidently doesn't realize this throughout his live-coding; at one point somebody in chat suggested to implement hard mode, to which he replied "maybe later" [TODO: timestamp on video] but then he went ahead and implemented a hard-mode-only solver. It's important to point out that 3blue1brown doesn't propose to implement a hard-mode solver; every guess is chosen to be one which gives the maximum expected information out of all the allowed words, not just the words that haven't been ruled out to be the answer [TODO: timestamp on video].

Jon went on to use memoization technique to speed up his program, but his assumptions are incorrect, which means his program no longer makes optimal guesses. Fortunately for him (or *unfortunately* in that it's hard to realize that something went wrong), making poor guesses in hard mode doesn't hurt that much. (In general, even in normal mode, the difference between Wordle solvers in terms of guess optimality is very slight. One might start out with 3.7 average guesses per game, and then cheat by overfitting the parameters on the actual set of anwers, only to get down to 3.6 average guesses per game.)

# On the solvers

Following the structure in the early days of roget [TODO: link to old version of roget with solvers in separate files], this repo offers multiple solvers, each one building on top of another by making it more efficient. The choice of guesses remains the same.
1. **naive.rs**: This is the first one. It is almost identical to the naive solver in roget, except that this one works in normal mode (which means that there are some additional complications in how to handle the situation of few possibilities left).
2. **cached.rs**: This one is like *allocs.rs* in roget (not *cache.rs*, which does unsound memoization). It modifies *naive.rs* by constructing the dictionary's HashMap once and then cloning it for subsequent games. There is barely any speedup.
3. **mask-buckets.rs**: An exponential speedup is introduced by decoupling the for loops iterating through candidate guesses and correctness patterns, taking advantage of the fact that every guess-answer pair corresponds to only one correctness pattern.
4. **memoized.rs**: Because the first guess is always the same, the second guess corresponding to each correctness pattern received for the first guess can be reused instead of being recomputed. Even on only 60 games, this shows 2x speedup compared to the previous solver. (The memoization is constructed along the way based on the patterns encountered throughout the 60 games; nothing is precomputed.) For more games, the speedup should be much higher.

# TODO

1. Implement hard mode solver.
2. Use sigmoid instead of bare frequency for modelling the probability distribution of the answer. (Because using bare frequency is so far off, the performance of using these solvers with normal mode is actually slightly worse than hard mode for the official list of Wordle answers.)
3. Implement interactive mode, where the program works as helper for somebody playing Wordle somewhere else. It should be able to accomodate the user telling that a word is not allowed, arbitrary history of previous guesses (not just the ones that the program would choose), and displaying a list of most-recommended guesses instead of just 1.
4. Try to memoize on arbitrarily long history of guesses, instead of just the second guess and assuming a hard-coded first guess. It should be a giant HashMap that stores the guesses made in the games that occured so far (not all possible games, which is astronomically big). Maintaining such a giant growing HashMap will have performance cost, but it should be worth it. There should be an option to save it to a file.
5. Decouple the server and the solver as separate concurrent programs, which then enables 100% efficient parallelization by simply having multiple solvers running at once on different games provided by the server.
6. Use the end game strategy discussed by 3blue1brown [TODO: timestamp on video] by letting statistics of previous performance give estimation of expected number of guesses left. Overfitting on only the official list of Wordle answers, however, is despised.

# License

This work is licensed under the [MIT License](http://opensource.org/licenses/MIT).

# Acknowledgment

This work is based on [roget](https://github.com/jonhoo/roget), with dramatic twists of its own, following in the original vision but rebuilding it to make it better. The theoretical basis, based on information theory, is by [3blue1brown](https://www.youtube.com/@3blue1brown).
