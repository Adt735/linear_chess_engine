# linear_chess_engine
A completely functional change engine with a well-optimized search function, whose eval funciton is just a linear model, where the coefficients are impored from a file.

# Build
If you have cargo installed, just run `cargo build --release` to obtain the rust executable

# Use
* To open the engine, run the executable with the following command line: `uci -i "path_to_your_coefficients_file"`
* Given a dataset with FEN and an evalutation, if you want to convert the FEN to a board representacion, run the executbale with: `process-csv -i "path_to_input_file" -o "path_to_output_file"`
