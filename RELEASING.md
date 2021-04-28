# Releasing

1. Update the `CHANGELOG.md`:
   1. Change the `Unreleased` header to the release version.
   2. Add a link URL to ensure the header link works.
   3. Add a new `Unreleased` section to the top.

2. Update `Cargo.toml` with the new version.

3. Commit

   ```
   $ git commit -am "Prepare version X.Y.X"
   ```

4. Tag

   ```
   $ git tag -am "Version X.Y.Z" X.Y.Z
   ```

5. Push!

   ```
   $ git push && git push --tags
   ```

   This will trigger a GitHub Action workflow which will create a GitHub release and
   automatically publish to Cargo, Docker Hub, and GitHub Container Registry.
