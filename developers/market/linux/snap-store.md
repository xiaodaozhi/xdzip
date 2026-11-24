# Snap Store publishing and listing

English | [简体中文](../../zh-CN/market/linux/snap-store.md)

Use this workflow for XDZip's `xdzip` listing and its Ubuntu 24.04 amd64 strict Snap. Use the [marketing export](README.md) for the other Linux presentation assets. Keep the version, source commit, package SHA-256, Store revision, actual checks and channel change in the release issue.

## Prepare and publish a revision

Dispatch [Release](https://github.com/xiaodaozhi/xdzip/actions/workflows/release.yml) on `main` with a stable tag using the [formal release procedure](../../engineering/release-runbook.md). The Linux job uses the executable and Qt SDK from the same native build to produce the strict amd64 Snap. It checks `core24`, pinned dependencies, packaged ELF loading and offscreen startup. The approved `release-signing` publication job publishes the GitHub Release, then uploads the Snap directly to `latest/stable`. It verifies the Store download hash and records the revision, source commit and SHA-256. Candidate and prerelease runs skip Store publication.

Store `SNAPCRAFT_STORE_CREDENTIALS` in the GitHub `release-signing` environment. Export it with `--snaps=xdzip --channels=latest/stable --acls=package_access,package_push,package_release` and set an expiry date. Use `gh secret set SNAPCRAFT_STORE_CREDENTIALS --env release-signing --repo xiaodaozhi/xdzip` with the credential file on standard input. [Credential export options](https://ubuntu.com/docs/snapcraft/latest/reference/commands/export-login/)

If the terminal already has an exported credential, begin the export command with `env -u SNAPCRAFT_STORE_CREDENTIALS snapcraft export-login` so Snapcraft can authenticate again.

### Manual release

1. Freeze the approved source revision and prepare the package using the [Snap verification procedure](../../qa/README.md#snap-candidate-runtime-dependencies). Use the existing Linux deployment and Snap assembly scripts. Check the installed entry point, base dependencies, manifest, notices and corresponding-source access for this exact package. [Release review](../../qa/release-review.md)
2. Upload the checked `.snap` with `snapcraft upload <path-to-snap> --release=latest/stable`. If Canonical requests manual review, check the [Releases tab](https://snapcraft.io/xdzip/releases). After review, a retry of the publication job recognizes the exact package if it is already public. [Canonical upload command](https://ubuntu.com/docs/snapcraft/9/reference/commands/upload/)
3. If the package was uploaded and remains unreleased, use `snapcraft revisions xdzip` to identify it and `snapcraft release xdzip <revision> latest/stable` to publish it. Confirm the public result with `snap info xdzip`. If a release must be withdrawn, review channel fallback and installed users before changing channels. [Canonical release command](https://ubuntu.com/docs/snapcraft/9/reference/commands/release/)

## Prepare copy and media

The public publisher form has a single title, summary, description and media collection per snap. XDZip uses English (`en-US`) for the shared Listing and links to the [multilingual product documentation](../../../docs/README.md) for other languages. Keep approved translations and localized promotional material in the existing ten-language [Linux marketing configuration](../../../specs/marketing/linux-store.json) and its export. A change in the Store's locale support should be verified in the publisher form and documented API before adding a per-language submission workflow. [Canonical listing API](https://dashboard.snapcraft.io/docs/reference/v1/snap.html#managing-snap-metadata)

Use the current approved product copy. The summary should fit 79 characters and the description should describe formats and capabilities in the distributed Snap. Update the website, source, support and privacy links as needed and check that users can reach the corresponding-source instructions. Preserve reviewed manual copy when releasing another binary revision.

| Publisher field | XDZip submission material |
| --- | --- |
| Snap icon | Export the project icon as a 256 × 256 PNG. The publisher form accepts square icons from 40 to 512 pixels per side, up to 256,000 bytes. |
| Images | Upload up to five verified Linux screenshots with sample data. Use an opening view first and then show core tasks. The publisher form accepts dimensions from 480 × 480 through 4320 × 2160, aspect ratios from 1:2 to 2:1 and up to 2,000,000 bytes per image. |
| Featured banner | Upload the shared 1440 × 480 PNG from `snap-store/featured-banner.png`. The publisher form accepts PNG or JPEG at 3:1, from 720 × 240 through 4320 × 1440, up to 2,000,000 bytes. Store featuring is handled by Canonical. |

The image limits come from the current publisher form. Review its image restrictions whenever preparing a submission. [Canonical listing and screenshot form](https://github.com/canonical/snapcraft.io/tree/main/static/js/publisher/pages/Listing/ListingDetails)

```sh
node scripts/build-linux-store-assets.cjs --output build/linux-store-assets
```

See [capture and export](../../engineering/platform-marketing.md#export-and-review) for prerequisites and how to attach newly verified native Linux captures. The exporter produces localized `text/` directories, branding, a shared Featured banner and a hash manifest. Upload native app images as screenshots. Use rendered brand artwork in the Featured banner slot.

## Save and verify the Listing

Open [XDZip Listing](https://snapcraft.io/xdzip/listing) from the publisher dashboard after the first revision is available. Fill the listing details and links, upload the icon and images, set the intended order and preview the layout. Save, reload the page and check the actual copy and image previews. Once a public channel exists, check the [public listing](https://snapcraft.io/xdzip) and the intended channel. The Listing and Releases tabs have separate save operations.

The Listing can be maintained through the publisher dashboard or Canonical's [metadata and binary-metadata APIs](https://dashboard.snapcraft.io/docs/reference/v1/snap.html#managing-snap-metadata). `snapcraft upload-metadata` can update supported fields from the package. Before using it on a hand-edited Listing, check what it would replace. Reuse unchanged media across releases.
