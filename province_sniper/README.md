# Province Sniper

Province sniper's main purpose is to read a provinces file (`definition.csv`) and based on either a `provinces.txt` or
`colors.txt` file, will remove provinces from the `definition.csv` if they match that criteria. An `error.log` or
`error.txt` may also be provided, and any errors in the form of `Province <id> has no pixels in provinces.bmp` will be
removed from `definition.csv`.

Running with `--inverse` will make province sniper remove provinces NOT defined in whatever file is provided.
Running with `--collapse` will just remove any "gaps" in province IDs.
