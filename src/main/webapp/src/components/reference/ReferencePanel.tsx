import styles from './ReferencePanel.module.css';

const example = `Theorem: "Fundamental Theorem of Calculus"
given: f(x), F(x), I, a, b
when:
. 'a, b is \\real'
. 'a < b'
. 'I := \\open.interval{a, b}'
. 'f(x) is \\continuous.function:on{I}'
. 'F(x) is \\indefinite.integral:of{f(x)}:on{I}'
then:
. '\\definite.integral[x]_{a}^{b}:of{f(x)} = F(b) - F(a)'
using:
. 'x - y := x \\real.-/ y'
. 'x < y := x \\real.lt/ y'`;

export const ReferencePanel = () => {
  return (
    <div className={styles.referenceBody}>
      <div className={styles.referenceContent}>
        <div className={styles.referenceHeader1}>The MathLingua Reference</div>
        <p>
          The MathLingua language consists of two parts, a structural language
          and an expression language as shown in the example below. For more
          information, see the language references below or visit&nbsp;
          <a href="https://www.mathlingua.org">mathlingua.org</a>.
        </p>
        <pre>{example}</pre>
        <div className={styles.referenceHeader1}>Expression Language</div>
        <p>
          The expression language is written within single quotes and is used to
          express mathematical statements.
        </p>
        <a href="#/help/expressionLanguage">Expression Language Reference</a>
        <div className={styles.referenceHeader1}>Structural Language</div>
        <p>
          The structural language, outside single quote statements, describes
          the overall structure of a mathematical construct.
        </p>
        <a href="#/help/structuralLanguage">Structural Language Reference</a>
      </div>
    </div>
  );
};
