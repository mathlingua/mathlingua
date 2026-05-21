import ViewerPage from "../viewer-page";

/** Ensures every deep viewer route reads the current CLI-generated payload. */
export const dynamic = "force-dynamic";

/** Catch-all viewer route for file and directory paths. */
export default ViewerPage;
