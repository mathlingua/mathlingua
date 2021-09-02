import RenderedComponent from '../rendered-component/RenderedComponent';
import styles from './BlockComment.module.css';

export interface BlockCommentProps {
  rawHtml: string;
  renderedHtml: string;
}

export const BlockComment = (props: BlockCommentProps) => {
  /*
   * A bug is causing the renderedHtml to contain class names like
   *   class=mathlingua - top - level
   * instead of the correct form
   *   class="mathlingua-top-level"
   * The following converts the first incorrect form to the second
   * correct form.
   */
  const fixedHtml = props.renderedHtml.replace(
    // Identify any `class=...` where the class= is not followed by
    // single or double quotes.  This uses the fact that mathlingua
    // adds class names that only contains alphanumeric characters
    // or the - character.
    /class[ ]*=[ ]*([ a-zA-Z0-9_-]+)/g,
    (m) => {
      // Now `m` contains the `class=...` string.
      // First replace any ` - ` with `-` (i.e. remove the spaces).
      // Next enclose the right-hand-side of `class=...` in quotes.
      return m.replace(/ - /g, '-').replace(/class=(.*)/g, 'class="$1"');
    }
  );
  return (
    <div
      className={styles.mathlinguaBlockCommentTopLevel}
      dangerouslySetInnerHTML={{
        __html: fixedHtml,
      }}
    ></div>
  );
};
