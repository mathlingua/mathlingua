if (typeof kotlin === 'undefined') {
  throw new Error("Error loading module 'bundle'. Its dependency 'kotlin' was not found. Please, check whether 'kotlin' is loaded prior to 'bundle'.");
}
var bundle = function (_, Kotlin) {
  'use strict';
  var Kind_CLASS = Kotlin.Kind.CLASS;
  var Unit = Kotlin.kotlin.Unit;
  var ArrayList_init = Kotlin.kotlin.collections.ArrayList_init_287e2$;
  var LinkedHashSet_init = Kotlin.kotlin.collections.LinkedHashSet_init_287e2$;
  var copyToArray = Kotlin.kotlin.collections.copyToArray;
  var RuntimeException_init = Kotlin.kotlin.RuntimeException_init_pdl1vj$;
  var RuntimeException = Kotlin.kotlin.RuntimeException;
  var IllegalArgumentException_init = Kotlin.kotlin.IllegalArgumentException_init_pdl1vj$;
  var Kind_OBJECT = Kotlin.Kind.OBJECT;
  var Kind_INTERFACE = Kotlin.Kind.INTERFACE;
  var endsWith = Kotlin.kotlin.text.endsWith_7epoxm$;
  var ensureNotNull = Kotlin.ensureNotNull;
  var toBoxedChar = Kotlin.toBoxedChar;
  var contains = Kotlin.kotlin.text.contains_sgbm27$;
  var Regex_init = Kotlin.kotlin.text.Regex_init_61zpoe$;
  var listOf = Kotlin.kotlin.collections.listOf_mh5how$;
  var toString = Kotlin.toString;
  var StringBuilder_init = Kotlin.kotlin.text.StringBuilder_init;
  var Enum = Kotlin.kotlin.Enum;
  var throwISE = Kotlin.throwISE;
  var equals = Kotlin.equals;
  var getCallableRef = Kotlin.getCallableRef;
  var listOf_0 = Kotlin.kotlin.collections.listOf_i5x0yv$;
  var throwCCE = Kotlin.throwCCE;
  var first = Kotlin.kotlin.collections.first_7wnvza$;
  var emptySet = Kotlin.kotlin.collections.emptySet_287e2$;
  var collectionSizeOrDefault = Kotlin.kotlin.collections.collectionSizeOrDefault_ba2ldo$;
  var ArrayList_init_0 = Kotlin.kotlin.collections.ArrayList_init_ww73n8$;
  var HashMap_init = Kotlin.kotlin.collections.HashMap_init_q3lmfv$;
  var Iterable = Kotlin.kotlin.collections.Iterable;
  var StringBuilder = Kotlin.kotlin.text.StringBuilder;
  var emptyList = Kotlin.kotlin.collections.emptyList_287e2$;
  var HashSet_init = Kotlin.kotlin.collections.HashSet_init_287e2$;
  ParseError.prototype = Object.create(RuntimeException.prototype);
  ParseError.prototype.constructor = ParseError;
  ChalkTalkTokenType.prototype = Object.create(Enum.prototype);
  ChalkTalkTokenType.prototype.constructor = ChalkTalkTokenType;
  TupleItem.prototype = Object.create(ChalkTalkTarget.prototype);
  TupleItem.prototype.constructor = TupleItem;
  AssignmentRhs.prototype = Object.create(TupleItem.prototype);
  AssignmentRhs.prototype.constructor = AssignmentRhs;
  ChalkTalkToken.prototype = Object.create(AssignmentRhs.prototype);
  ChalkTalkToken.prototype.constructor = ChalkTalkToken;
  Mapping.prototype = Object.create(ChalkTalkTarget.prototype);
  Mapping.prototype.constructor = Mapping;
  Group.prototype = Object.create(ChalkTalkTarget.prototype);
  Group.prototype.constructor = Group;
  Assignment.prototype = Object.create(TupleItem.prototype);
  Assignment.prototype.constructor = Assignment;
  Tuple.prototype = Object.create(AssignmentRhs.prototype);
  Tuple.prototype.constructor = Tuple;
  Abstraction.prototype = Object.create(TupleItem.prototype);
  Abstraction.prototype.constructor = Abstraction;
  Aggregate.prototype = Object.create(AssignmentRhs.prototype);
  Aggregate.prototype.constructor = Aggregate;
  Target.prototype = Object.create(Clause.prototype);
  Target.prototype.constructor = Target;
  AbstractionNode.prototype = Object.create(Target.prototype);
  AbstractionNode.prototype.constructor = AbstractionNode;
  AggregateNode.prototype = Object.create(Target.prototype);
  AggregateNode.prototype.constructor = AggregateNode;
  TupleNode.prototype = Object.create(Target.prototype);
  TupleNode.prototype.constructor = TupleNode;
  AssignmentNode.prototype = Object.create(Target.prototype);
  AssignmentNode.prototype.constructor = AssignmentNode;
  Identifier.prototype = Object.create(Target.prototype);
  Identifier.prototype.constructor = Identifier;
  Statement.prototype = Object.create(Clause.prototype);
  Statement.prototype.constructor = Statement;
  Text.prototype = Object.create(Clause.prototype);
  Text.prototype.constructor = Text;
  ExistsGroup.prototype = Object.create(Clause.prototype);
  ExistsGroup.prototype.constructor = ExistsGroup;
  IfGroup.prototype = Object.create(Clause.prototype);
  IfGroup.prototype.constructor = IfGroup;
  IffGroup.prototype = Object.create(Clause.prototype);
  IffGroup.prototype.constructor = IffGroup;
  ForGroup.prototype = Object.create(Clause.prototype);
  ForGroup.prototype.constructor = ForGroup;
  NotGroup.prototype = Object.create(Clause.prototype);
  NotGroup.prototype.constructor = NotGroup;
  OrGroup.prototype = Object.create(Clause.prototype);
  OrGroup.prototype.constructor = OrGroup;
  NodeType.prototype = Object.create(Enum.prototype);
  NodeType.prototype.constructor = NodeType;
  TexTalkTokenType.prototype = Object.create(Enum.prototype);
  TexTalkTokenType.prototype.constructor = TexTalkTokenType;
  function MathLinguaResult(document, errors) {
    this.document = document;
    this.errors = errors;
  }
  MathLinguaResult.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'MathLinguaResult',
    interfaces: []
  };
  MathLinguaResult.prototype.component1 = function () {
    return this.document;
  };
  MathLinguaResult.prototype.component2 = function () {
    return this.errors;
  };
  MathLinguaResult.prototype.copy_aig4j4$ = function (document, errors) {
    return new MathLinguaResult(document === void 0 ? this.document : document, errors === void 0 ? this.errors : errors);
  };
  MathLinguaResult.prototype.toString = function () {
    return 'MathLinguaResult(document=' + Kotlin.toString(this.document) + (', errors=' + Kotlin.toString(this.errors)) + ')';
  };
  MathLinguaResult.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.document) | 0;
    result = result * 31 + Kotlin.hashCode(this.errors) | 0;
    return result;
  };
  MathLinguaResult.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.document, other.document) && Kotlin.equals(this.errors, other.errors)))));
  };
  function MathLingua() {
  }
  MathLingua.prototype.parse_61zpoe$ = function (input) {
    var lexer = newChalkTalkLexer(input);
    var allErrors = ArrayList_init();
    allErrors.addAll_brywnq$(lexer.errors());
    var parser = newChalkTalkParser();
    var tmp$ = parser.parse_khrmll$(lexer);
    var root = tmp$.component1()
    , errors = tmp$.component2();
    allErrors.addAll_brywnq$(errors);
    if (root == null) {
      return new MathLinguaResult(null, allErrors);
    }
    var documentValidation = Document$Companion_getInstance().validate_rk66c5$(root);
    allErrors.addAll_brywnq$(documentValidation.errors);
    return new MathLinguaResult(documentValidation.value, allErrors);
  };
  MathLingua.prototype.findAllSignatures_mu0sga$ = function (node) {
    var signatures = LinkedHashSet_init();
    findAllSignaturesImpl(node, signatures);
    return copyToArray(signatures);
  };
  MathLingua.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'MathLingua',
    interfaces: []
  };
  function findAllSignaturesImpl$lambda(closure$signatures) {
    return function (it) {
      findAllSignaturesImpl(it, closure$signatures);
      return Unit;
    };
  }
  function findAllSignaturesImpl(node, signatures) {
    if (Kotlin.isType(node, Statement)) {
      var sigs = findAllStatementSignatures(node);
      signatures.addAll_brywnq$(sigs);
    }
    node.forEach_ye21ev$(findAllSignaturesImpl$lambda(signatures));
  }
  function ParseError(message, row, column) {
    RuntimeException_init(message, this);
    this.message_rj2t0z$_0 = message;
    this.row = row;
    this.column = column;
    this.name = 'ParseError';
  }
  Object.defineProperty(ParseError.prototype, 'message', {
    get: function () {
      return this.message_rj2t0z$_0;
    }
  });
  ParseError.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ParseError',
    interfaces: [RuntimeException]
  };
  ParseError.prototype.component1 = function () {
    return this.message;
  };
  ParseError.prototype.component2 = function () {
    return this.row;
  };
  ParseError.prototype.component3 = function () {
    return this.column;
  };
  ParseError.prototype.copy_3m52m6$ = function (message, row, column) {
    return new ParseError(message === void 0 ? this.message : message, row === void 0 ? this.row : row, column === void 0 ? this.column : column);
  };
  ParseError.prototype.toString = function () {
    return 'ParseError(message=' + Kotlin.toString(this.message) + (', row=' + Kotlin.toString(this.row)) + (', column=' + Kotlin.toString(this.column)) + ')';
  };
  ParseError.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.message) | 0;
    result = result * 31 + Kotlin.hashCode(this.row) | 0;
    result = result * 31 + Kotlin.hashCode(this.column) | 0;
    return result;
  };
  ParseError.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.message, other.message) && Kotlin.equals(this.row, other.row) && Kotlin.equals(this.column, other.column)))));
  };
  function Validation(value, errors) {
    Validation$Companion_getInstance();
    this.value = value;
    this.errors = errors;
  }
  Object.defineProperty(Validation.prototype, 'isSuccessful', {
    get: function () {
      return this.value != null;
    }
  });
  function Validation$Companion() {
    Validation$Companion_instance = this;
  }
  Validation$Companion.prototype.success_mh5how$ = function (value) {
    if (value == null) {
      throw IllegalArgumentException_init('A successful validation cannot have a null value');
    }
    return new Validation(value, ArrayList_init());
  };
  Validation$Companion.prototype.failure_rg4ulb$ = function (errors) {
    return new Validation(null, errors);
  };
  Validation$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var Validation$Companion_instance = null;
  function Validation$Companion_getInstance() {
    if (Validation$Companion_instance === null) {
      new Validation$Companion();
    }
    return Validation$Companion_instance;
  }
  Validation.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Validation',
    interfaces: []
  };
  function ChalkTalkLexer() {
  }
  ChalkTalkLexer.$metadata$ = {
    kind: Kind_INTERFACE,
    simpleName: 'ChalkTalkLexer',
    interfaces: []
  };
  function newChalkTalkLexer(text) {
    return new ChalkTalkLexerImpl(text);
  }
  function ChalkTalkLexerImpl(text) {
    this.text_0 = text;
    this.errors_0 = null;
    this.chalkTalkTokens_0 = null;
    this.index_0 = 0;
    this.errors_0 = ArrayList_init();
    this.chalkTalkTokens_0 = null;
    this.index_0 = 0;
  }
  ChalkTalkLexerImpl.prototype.ensureInitialized_0 = function () {
    if (this.chalkTalkTokens_0 == null) {
      this.initialize_0();
    }
  };
  ChalkTalkLexerImpl.prototype.initialize_0 = function () {
    var tmp$, tmp$_0, tmp$_1, tmp$_2, tmp$_3, tmp$_4, tmp$_5, tmp$_6, tmp$_7, tmp$_8, tmp$_9, tmp$_10, tmp$_11, tmp$_12, tmp$_13, tmp$_14, tmp$_15, tmp$_16, tmp$_17, tmp$_18, tmp$_19, tmp$_20, tmp$_21, tmp$_22, tmp$_23, tmp$_24, tmp$_25;
    this.chalkTalkTokens_0 = ArrayList_init();
    if (!endsWith(this.text_0, '\n')) {
      this.text_0 = this.text_0 + '\n';
    }
    var i = 0;
    var line = 0;
    var column = -1;
    var levStack = new Stack();
    levStack.push_11rb$(0);
    var numOpen = 0;
    while (i < this.text_0.length) {
      if (this.text_0.charCodeAt(i) === 45 && (i + 1 | 0) < this.text_0.length && this.text_0.charCodeAt(i + 1 | 0) === 45) {
        while (i < this.text_0.length && this.text_0.charCodeAt(i) !== 10) {
          i = i + 1 | 0;
          column = column + 1 | 0;
        }
        if (i < this.text_0.length && this.text_0.charCodeAt(i) === 10) {
          i = i + 1 | 0;
          column = 0;
          line = line + 1 | 0;
        }
        continue;
      }
      var c = this.text_0.charCodeAt((tmp$ = i, i = tmp$ + 1 | 0, tmp$));
      column = column + 1 | 0;
      if (c === 61) {
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken('=', ChalkTalkTokenType$Equals_getInstance(), line, column));
      }
       else if (c === 40) {
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken('(', ChalkTalkTokenType$LParen_getInstance(), line, column));
      }
       else if (c === 41) {
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken(')', ChalkTalkTokenType$RParen_getInstance(), line, column));
      }
       else if (c === 123) {
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken('{', ChalkTalkTokenType$LCurly_getInstance(), line, column));
      }
       else if (c === 125) {
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken('}', ChalkTalkTokenType$RCurly_getInstance(), line, column));
      }
       else if (c === 58) {
        if (i < this.text_0.length && this.text_0.charCodeAt(i) === 61) {
          ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken(':=', ChalkTalkTokenType$ColonEquals_getInstance(), line, column));
          i = i + 1 | 0;
        }
         else {
          ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken(':', ChalkTalkTokenType$Colon_getInstance(), line, column));
        }
      }
       else if (c === 44) {
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken(',', ChalkTalkTokenType$Comma_getInstance(), line, column));
      }
       else if (c === 46 && i < this.text_0.length && this.text_0.charCodeAt(i) === 32) {
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken('. ', ChalkTalkTokenType$DotSpace_getInstance(), line, column));
        i = i + 1 | 0;
        column = column + 1 | 0;
      }
       else if (c === 10) {
        line = line + 1 | 0;
        column = 0;
        if ((i - 2 | 0) < 0 || this.text_0.charCodeAt(i - 2 | 0) === 10) {
          while (i < this.text_0.length && this.text_0.charCodeAt(i) === 10) {
            i = i + 1 | 0;
            column = column + 1 | 0;
            line = line + 1 | 0;
          }
          ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken('-', ChalkTalkTokenType$Linebreak_getInstance(), line, column));
          continue;
        }
        var indentCount = 0;
        while (i < this.text_0.length && (i + 1 | 0) < this.text_0.length && this.text_0.charCodeAt(i) === 32 && this.text_0.charCodeAt(i + 1 | 0) === 32) {
          indentCount = indentCount + 1 | 0;
          i = i + 2 | 0;
          column = column + 2 | 0;
        }
        if (i < this.text_0.length && this.text_0.charCodeAt(i) === 46 && (i + 1 | 0) < this.text_0.length && this.text_0.charCodeAt(i + 1 | 0) === 32) {
          indentCount = indentCount + 1 | 0;
        }
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken('<Indent>', ChalkTalkTokenType$Begin_getInstance(), line, column));
        numOpen = numOpen + 1 | 0;
        var level = levStack.peek();
        if (indentCount <= level) {
          while (numOpen > 0 && !levStack.isEmpty() && indentCount <= levStack.peek()) {
            ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken('<Unindent>', ChalkTalkTokenType$End_getInstance(), line, column));
            numOpen = numOpen - 1 | 0;
            levStack.pop();
          }
          if (levStack.isEmpty()) {
            levStack.push_11rb$(0);
          }
        }
        levStack.push_11rb$(indentCount);
      }
       else if (this.isOperatorChar_0(c)) {
        var name = '' + String.fromCharCode(toBoxedChar(c));
        while (i < this.text_0.length && this.isOperatorChar_0(this.text_0.charCodeAt(i))) {
          tmp$_3 = name;
          tmp$_2 = this.text_0;
          tmp$_1 = (tmp$_0 = i, i = tmp$_0 + 1 | 0, tmp$_0);
          name = tmp$_3 + String.fromCharCode(tmp$_2.charCodeAt(tmp$_1));
          column = column + 1 | 0;
        }
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken(name, ChalkTalkTokenType$Name_getInstance(), line, column));
      }
       else if (this.isLetterOrDigit_0(c)) {
        var name_0 = '' + String.fromCharCode(toBoxedChar(c));
        while (i < this.text_0.length && this.isLetterOrDigit_0(this.text_0.charCodeAt(i))) {
          tmp$_7 = name_0;
          tmp$_6 = this.text_0;
          tmp$_5 = (tmp$_4 = i, i = tmp$_4 + 1 | 0, tmp$_4);
          name_0 = tmp$_7 + String.fromCharCode(tmp$_6.charCodeAt(tmp$_5));
          column = column + 1 | 0;
        }
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken(name_0, ChalkTalkTokenType$Name_getInstance(), line, column));
      }
       else if (c === 34) {
        var str = '' + String.fromCharCode(toBoxedChar(c));
        while (i < this.text_0.length && this.text_0.charCodeAt(i) !== 34) {
          tmp$_11 = str;
          tmp$_10 = this.text_0;
          tmp$_9 = (tmp$_8 = i, i = tmp$_8 + 1 | 0, tmp$_8);
          str = tmp$_11 + String.fromCharCode(tmp$_10.charCodeAt(tmp$_9));
          column = column + 1 | 0;
        }
        if (i === this.text_0.length) {
          this.errors_0.add_11rb$(new ParseError('Expected a terminating "', line, column));
          str += '"';
        }
         else {
          tmp$_15 = str;
          tmp$_14 = this.text_0;
          tmp$_13 = (tmp$_12 = i, i = tmp$_12 + 1 | 0, tmp$_12);
          str = tmp$_15 + String.fromCharCode(tmp$_14.charCodeAt(tmp$_13));
          column = column + 1 | 0;
        }
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken(str, ChalkTalkTokenType$String_getInstance(), line, column));
      }
       else if (c === 39) {
        var stmt = '' + String.fromCharCode(toBoxedChar(c));
        while (i < this.text_0.length && this.text_0.charCodeAt(i) !== 39) {
          tmp$_19 = stmt;
          tmp$_18 = this.text_0;
          tmp$_17 = (tmp$_16 = i, i = tmp$_16 + 1 | 0, tmp$_16);
          stmt = tmp$_19 + String.fromCharCode(tmp$_18.charCodeAt(tmp$_17));
          column = column + 1 | 0;
        }
        if (i === this.text_0.length) {
          this.errors_0.add_11rb$(new ParseError("Expected a terminating '", line, column));
          stmt += "'";
        }
         else {
          tmp$_23 = stmt;
          tmp$_22 = this.text_0;
          tmp$_21 = (tmp$_20 = i, i = tmp$_20 + 1 | 0, tmp$_20);
          stmt = tmp$_23 + String.fromCharCode(tmp$_22.charCodeAt(tmp$_21));
          column = column + 1 | 0;
        }
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken(stmt, ChalkTalkTokenType$Statement_getInstance(), line, column));
      }
       else if (c === 91) {
        var startLine = line;
        var startColumn = column;
        var id = '' + String.fromCharCode(toBoxedChar(c));
        var braceCount = 1;
        while (i < this.text_0.length && this.text_0.charCodeAt(i) !== 10) {
          var next = this.text_0.charCodeAt((tmp$_24 = i, i = tmp$_24 + 1 | 0, tmp$_24));
          id += String.fromCharCode(next);
          column = column + 1 | 0;
          if (next === 91) {
            braceCount = braceCount + 1 | 0;
          }
           else if (next === 93) {
            tmp$_25 = braceCount, braceCount = tmp$_25 - 1 | 0;
          }
          if (braceCount === 0) {
            break;
          }
        }
        if (i === this.text_0.length) {
          this.errors_0.add_11rb$(new ParseError('Expected a terminating ]', line, column));
          id += ']';
        }
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken(id, ChalkTalkTokenType$Id_getInstance(), startLine, startColumn));
      }
       else if (c !== 32) {
        this.errors_0.add_11rb$(new ParseError('Unrecognized character ' + String.fromCharCode(c), line, column));
      }
    }
    while (numOpen > 0) {
      ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new ChalkTalkToken('<Unindent>', ChalkTalkTokenType$End_getInstance(), line, column));
      numOpen = numOpen - 1 | 0;
    }
  };
  ChalkTalkLexerImpl.prototype.isOperatorChar_0 = function (c) {
    return contains('~!@#%^&*-+<>\\/', c);
  };
  ChalkTalkLexerImpl.prototype.hasNext = function () {
    this.ensureInitialized_0();
    return this.index_0 < ensureNotNull(this.chalkTalkTokens_0).size;
  };
  ChalkTalkLexerImpl.prototype.hasNextNext = function () {
    this.ensureInitialized_0();
    return (this.index_0 + 1 | 0) < ensureNotNull(this.chalkTalkTokens_0).size;
  };
  ChalkTalkLexerImpl.prototype.peek = function () {
    this.ensureInitialized_0();
    return ensureNotNull(this.chalkTalkTokens_0).get_za3lpa$(this.index_0);
  };
  ChalkTalkLexerImpl.prototype.peekPeek = function () {
    this.ensureInitialized_0();
    return ensureNotNull(this.chalkTalkTokens_0).get_za3lpa$(this.index_0 + 1 | 0);
  };
  ChalkTalkLexerImpl.prototype.next = function () {
    this.ensureInitialized_0();
    var tok = this.peek();
    this.index_0 = this.index_0 + 1 | 0;
    return tok;
  };
  ChalkTalkLexerImpl.prototype.errors = function () {
    return this.errors_0;
  };
  ChalkTalkLexerImpl.prototype.isLetterOrDigit_0 = function (c) {
    return Regex_init('[a-zA-Z0-9]+').matches_6bul2c$(String.fromCharCode(c));
  };
  ChalkTalkLexerImpl.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ChalkTalkLexerImpl',
    interfaces: [ChalkTalkLexer]
  };
  function Stack() {
    this.data_0 = ArrayList_init();
  }
  Stack.prototype.push_11rb$ = function (item) {
    this.data_0.add_11rb$(item);
  };
  Stack.prototype.pop = function () {
    return this.data_0.removeAt_za3lpa$(this.data_0.size - 1 | 0);
  };
  Stack.prototype.peek = function () {
    var $receiver = this.data_0;
    var index = this.data_0.size - 1 | 0;
    return $receiver.get_za3lpa$(index);
  };
  Stack.prototype.isEmpty = function () {
    return this.data_0.isEmpty();
  };
  Stack.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Stack',
    interfaces: []
  };
  function ChalkTalkParser() {
  }
  ChalkTalkParser.$metadata$ = {
    kind: Kind_INTERFACE,
    simpleName: 'ChalkTalkParser',
    interfaces: []
  };
  function ChalkTalkParseResult(root, errors) {
    this.root = root;
    this.errors = errors;
  }
  ChalkTalkParseResult.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ChalkTalkParseResult',
    interfaces: []
  };
  ChalkTalkParseResult.prototype.component1 = function () {
    return this.root;
  };
  ChalkTalkParseResult.prototype.component2 = function () {
    return this.errors;
  };
  ChalkTalkParseResult.prototype.copy_vlv7gc$ = function (root, errors) {
    return new ChalkTalkParseResult(root === void 0 ? this.root : root, errors === void 0 ? this.errors : errors);
  };
  ChalkTalkParseResult.prototype.toString = function () {
    return 'ChalkTalkParseResult(root=' + Kotlin.toString(this.root) + (', errors=' + Kotlin.toString(this.errors)) + ')';
  };
  ChalkTalkParseResult.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.root) | 0;
    result = result * 31 + Kotlin.hashCode(this.errors) | 0;
    return result;
  };
  ChalkTalkParseResult.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.root, other.root) && Kotlin.equals(this.errors, other.errors)))));
  };
  function newChalkTalkParser() {
    return new ChalkTalkParserImpl();
  }
  function ChalkTalkParserImpl() {
  }
  ChalkTalkParserImpl.prototype.parse_khrmll$ = function (chalkTalkLexer) {
    var worker = new ChalkTalkParserImpl$ParserWorker(chalkTalkLexer);
    var errors = worker.errors;
    var root = worker.root();
    return new ChalkTalkParseResult(root, errors);
  };
  function ChalkTalkParserImpl$ParserWorker(chalkTalkLexer) {
    this.chalkTalkLexer_0 = chalkTalkLexer;
    this.errors = null;
    this.errors = ArrayList_init();
  }
  ChalkTalkParserImpl$ParserWorker.prototype.root = function () {
    var groups = ArrayList_init();
    while (true) {
      var grp = this.group_0();
      if (grp == null)
        break;
      groups.add_11rb$(grp);
    }
    while (this.chalkTalkLexer_0.hasNext()) {
      var next = this.chalkTalkLexer_0.next();
      this.errors.add_11rb$(new ParseError("Unrecognized token '" + next.text, next.row, next.column));
    }
    return new Root(groups);
  };
  ChalkTalkParserImpl$ParserWorker.prototype.group_0 = function () {
    if (this.chalkTalkLexer_0.hasNext() && this.chalkTalkLexer_0.peek().type === ChalkTalkTokenType$Linebreak_getInstance()) {
      this.chalkTalkLexer_0.next();
    }
    var id = null;
    if (this.chalkTalkLexer_0.hasNext() && this.chalkTalkLexer_0.peek().type === ChalkTalkTokenType$Id_getInstance()) {
      id = this.chalkTalkLexer_0.next();
      this.expect_0(ChalkTalkTokenType$Begin_getInstance());
      this.expect_0(ChalkTalkTokenType$End_getInstance());
    }
    var sections = ArrayList_init();
    while (true) {
      var sec = this.section_0();
      if (sec == null)
        break;
      sections.add_11rb$(sec);
    }
    return sections.isEmpty() ? null : new Group(sections, id);
  };
  ChalkTalkParserImpl$ParserWorker.prototype.section_0 = function () {
    var isSec = this.chalkTalkLexer_0.hasNext() && this.chalkTalkLexer_0.hasNextNext() && this.chalkTalkLexer_0.peek().type === ChalkTalkTokenType$Name_getInstance() && this.chalkTalkLexer_0.peekPeek().type === ChalkTalkTokenType$Colon_getInstance();
    if (!isSec) {
      return null;
    }
    var name = this.expect_0(ChalkTalkTokenType$Name_getInstance());
    this.expect_0(ChalkTalkTokenType$Colon_getInstance());
    var args = ArrayList_init();
    while (this.chalkTalkLexer_0.hasNext() && this.chalkTalkLexer_0.peek().type !== ChalkTalkTokenType$Begin_getInstance()) {
      var arg = this.argument_0();
      if (arg == null)
        break;
      args.add_11rb$(arg);
      if (this.chalkTalkLexer_0.hasNext() && this.chalkTalkLexer_0.peek().type !== ChalkTalkTokenType$Begin_getInstance()) {
        this.expect_0(ChalkTalkTokenType$Comma_getInstance());
      }
    }
    this.expect_0(ChalkTalkTokenType$Begin_getInstance());
    while (true) {
      var argList = this.argumentList_0();
      if (argList == null)
        break;
      args.addAll_brywnq$(argList);
    }
    this.expect_0(ChalkTalkTokenType$End_getInstance());
    return new Section(name, args);
  };
  ChalkTalkParserImpl$ParserWorker.prototype.argumentList_0 = function () {
    if (!this.chalkTalkLexer_0.hasNext() || this.chalkTalkLexer_0.peek().type !== ChalkTalkTokenType$DotSpace_getInstance()) {
      return null;
    }
    this.expect_0(ChalkTalkTokenType$DotSpace_getInstance());
    var grp = this.group_0();
    if (grp != null) {
      return listOf(new Argument(grp));
    }
    var argList = ArrayList_init();
    var valueArg = this.argument_0();
    if (valueArg != null) {
      argList.add_11rb$(valueArg);
      while (this.chalkTalkLexer_0.hasNext() && this.chalkTalkLexer_0.peek().type === ChalkTalkTokenType$Comma_getInstance()) {
        this.chalkTalkLexer_0.next();
        var v = this.argument_0();
        if (v == null)
          break;
        argList.add_11rb$(v);
      }
    }
    this.expect_0(ChalkTalkTokenType$Begin_getInstance());
    this.expect_0(ChalkTalkTokenType$End_getInstance());
    return !argList.isEmpty() ? argList : null;
  };
  ChalkTalkParserImpl$ParserWorker.prototype.token_0 = function (type) {
    var tmp$;
    if (this.has_0(type)) {
      tmp$ = this.chalkTalkLexer_0.next();
    }
     else {
      tmp$ = null;
    }
    return tmp$;
  };
  ChalkTalkParserImpl$ParserWorker.prototype.argument_0 = function () {
    var tmp$;
    var literal = (tmp$ = this.token_0(ChalkTalkTokenType$Statement_getInstance())) != null ? tmp$ : this.token_0(ChalkTalkTokenType$String_getInstance());
    if (literal != null) {
      return new Argument(literal);
    }
    var mapping = this.mapping_0();
    if (mapping != null) {
      return new Argument(mapping);
    }
    var target = this.tupleItem_0();
    if (target == null) {
      this.errors.add_11rb$(new ParseError('Expected a name, abstraction, tuple, aggregate, or assignment', -1, -1));
      var tok = new ChalkTalkToken('INVALID', ChalkTalkTokenType$Invalid_getInstance(), -1, -1);
      return new Argument(tok);
    }
    return new Argument(target);
  };
  ChalkTalkParserImpl$ParserWorker.prototype.mapping_0 = function () {
    if (!this.hasHas_0(ChalkTalkTokenType$Name_getInstance(), ChalkTalkTokenType$Equals_getInstance())) {
      return null;
    }
    var name = this.chalkTalkLexer_0.next();
    var equals = this.chalkTalkLexer_0.next();
    var rhs;
    if (!this.chalkTalkLexer_0.hasNext()) {
      this.errors.add_11rb$(new ParseError('A = must be followed by an argument', equals.row, equals.column));
      rhs = new ChalkTalkToken('INVALID', ChalkTalkTokenType$Invalid_getInstance(), -1, -1);
    }
     else {
      var maybeRhs = this.chalkTalkLexer_0.next();
      if (maybeRhs.type === ChalkTalkTokenType$String_getInstance()) {
        rhs = maybeRhs;
      }
       else {
        new ParseError('The right hand side of a = must be a string', equals.row, equals.column);
        rhs = new ChalkTalkToken('INVALID', ChalkTalkTokenType$Invalid_getInstance(), -1, -1);
      }
    }
    return new Mapping(name, rhs);
  };
  ChalkTalkParserImpl$ParserWorker.prototype.assignment_0 = function () {
    if (!this.hasHas_0(ChalkTalkTokenType$Name_getInstance(), ChalkTalkTokenType$ColonEquals_getInstance())) {
      return null;
    }
    var name = this.chalkTalkLexer_0.next();
    var colonEquals = this.chalkTalkLexer_0.next();
    var rhs = this.assignmentRhs_0();
    if (rhs == null) {
      this.errors.add_11rb$(new ParseError('A := must be followed by a argument', colonEquals.row, colonEquals.column));
      rhs = new ChalkTalkToken('INVALID', ChalkTalkTokenType$Invalid_getInstance(), -1, -1);
    }
    return new Assignment(name, rhs);
  };
  ChalkTalkParserImpl$ParserWorker.prototype.aggregate_0 = function () {
    if (!this.has_0(ChalkTalkTokenType$LCurly_getInstance())) {
      return null;
    }
    this.expect_0(ChalkTalkTokenType$LCurly_getInstance());
    var names = this.nameList_0(ChalkTalkTokenType$RCurly_getInstance());
    this.expect_0(ChalkTalkTokenType$RCurly_getInstance());
    return new Aggregate(names);
  };
  ChalkTalkParserImpl$ParserWorker.prototype.abstraction_0 = function () {
    if (!this.hasHas_0(ChalkTalkTokenType$Name_getInstance(), ChalkTalkTokenType$LParen_getInstance())) {
      return null;
    }
    var id = this.expect_0(ChalkTalkTokenType$Name_getInstance());
    this.expect_0(ChalkTalkTokenType$LParen_getInstance());
    var names = this.nameList_0(ChalkTalkTokenType$RParen_getInstance());
    this.expect_0(ChalkTalkTokenType$RParen_getInstance());
    return new Abstraction(id, names);
  };
  ChalkTalkParserImpl$ParserWorker.prototype.name_0 = function () {
    if (!this.has_0(ChalkTalkTokenType$Name_getInstance())) {
      if (this.chalkTalkLexer_0.hasNext()) {
        var peek = this.chalkTalkLexer_0.next();
        this.errors.add_11rb$(new ParseError('Expected a name, but found ' + peek.text, peek.row, peek.column));
      }
       else {
        this.errors.add_11rb$(new ParseError('Expected a name, but found the end of input', -1, -1));
      }
      return new ChalkTalkToken('', ChalkTalkTokenType$Invalid_getInstance(), -1, -1);
    }
    return this.chalkTalkLexer_0.next();
  };
  ChalkTalkParserImpl$ParserWorker.prototype.tuple_0 = function () {
    if (!this.has_0(ChalkTalkTokenType$LParen_getInstance())) {
      return null;
    }
    var items = ArrayList_init();
    var leftParen = this.expect_0(ChalkTalkTokenType$LParen_getInstance());
    while (this.chalkTalkLexer_0.hasNext() && this.chalkTalkLexer_0.peek().type !== ChalkTalkTokenType$RParen_getInstance()) {
      if (!items.isEmpty()) {
        this.expect_0(ChalkTalkTokenType$Comma_getInstance());
      }
      var item = this.tupleItem_0();
      if (item == null) {
        this.errors.add_11rb$(new ParseError('Encountered a non-tuple item in a tuple', leftParen.row, leftParen.column));
      }
       else {
        items.add_11rb$(item);
      }
    }
    this.expect_0(ChalkTalkTokenType$RParen_getInstance());
    return new Tuple(items);
  };
  ChalkTalkParserImpl$ParserWorker.prototype.assignmentRhs_0 = function () {
    var tmp$, tmp$_0;
    return (tmp$_0 = (tmp$ = this.tuple_0()) != null ? tmp$ : this.aggregate_0()) != null ? tmp$_0 : this.name_0();
  };
  ChalkTalkParserImpl$ParserWorker.prototype.tupleItem_0 = function () {
    var tmp$, tmp$_0;
    return (tmp$_0 = (tmp$ = this.assignment_0()) != null ? tmp$ : this.abstraction_0()) != null ? tmp$_0 : this.assignmentRhs_0();
  };
  ChalkTalkParserImpl$ParserWorker.prototype.nameList_0 = function (stopType) {
    var names = ArrayList_init();
    while (this.chalkTalkLexer_0.hasNext() && this.chalkTalkLexer_0.peek().type !== stopType) {
      var comma = null;
      if (!names.isEmpty()) {
        comma = this.expect_0(ChalkTalkTokenType$Comma_getInstance());
      }
      if (!this.chalkTalkLexer_0.hasNext()) {
        this.errors.add_11rb$(new ParseError('Expected a name to follow a comma', ensureNotNull(comma).row, comma.column));
        break;
      }
      var tok = this.chalkTalkLexer_0.next();
      if (tok.type === ChalkTalkTokenType$Name_getInstance()) {
        names.add_11rb$(tok);
      }
       else {
        this.errors.add_11rb$(new ParseError("Expected a name but found '" + tok.text + "'", tok.row, tok.column));
      }
    }
    return names;
  };
  ChalkTalkParserImpl$ParserWorker.prototype.has_0 = function (type) {
    return this.chalkTalkLexer_0.hasNext() && this.chalkTalkLexer_0.peek().type === type;
  };
  ChalkTalkParserImpl$ParserWorker.prototype.hasHas_0 = function (type, thenType) {
    return this.has_0(type) && this.chalkTalkLexer_0.hasNextNext() && this.chalkTalkLexer_0.peekPeek().type === thenType;
  };
  ChalkTalkParserImpl$ParserWorker.prototype.expect_0 = function (type) {
    var tmp$;
    if (!this.chalkTalkLexer_0.hasNext() || this.chalkTalkLexer_0.peek().type !== type) {
      if (this.chalkTalkLexer_0.hasNext()) {
        tmp$ = this.chalkTalkLexer_0.peek();
      }
       else {
        tmp$ = new ChalkTalkToken('', ChalkTalkTokenType$Invalid_getInstance(), -1, -1);
      }
      var peek = tmp$;
      this.errors.add_11rb$(new ParseError('Expected a token of type ' + toString(type) + ' but ' + 'found ' + toString(peek.type), peek.row, peek.column));
      return new ChalkTalkToken('INVALID', ChalkTalkTokenType$Invalid_getInstance(), -1, -1);
    }
    return this.chalkTalkLexer_0.next();
  };
  ChalkTalkParserImpl$ParserWorker.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ParserWorker',
    interfaces: []
  };
  ChalkTalkParserImpl.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ChalkTalkParserImpl',
    interfaces: [ChalkTalkParser]
  };
  function ChalkTalkNode() {
  }
  ChalkTalkNode.$metadata$ = {
    kind: Kind_INTERFACE,
    simpleName: 'ChalkTalkNode',
    interfaces: []
  };
  function Root(groups) {
    this.groups = groups;
  }
  Root.prototype.forEach_ewgams$ = function (fn) {
    var tmp$;
    tmp$ = this.groups.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  Root.prototype.print_irqrwq$ = function (buffer) {
    var tmp$;
    tmp$ = this.groups.iterator();
    while (tmp$.hasNext()) {
      var grp = tmp$.next();
      grp.print_yrfq27$(buffer, 0, false);
    }
  };
  Root.prototype.toCode = function () {
    var buffer = StringBuilder_init();
    this.print_irqrwq$(buffer);
    return buffer.toString();
  };
  Root.prototype.resolve = function () {
    return this;
  };
  Root.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Root',
    interfaces: [ChalkTalkNode]
  };
  Root.prototype.component1 = function () {
    return this.groups;
  };
  Root.prototype.copy_h9xwie$ = function (groups) {
    return new Root(groups === void 0 ? this.groups : groups);
  };
  Root.prototype.toString = function () {
    return 'Root(groups=' + Kotlin.toString(this.groups) + ')';
  };
  Root.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.groups) | 0;
    return result;
  };
  Root.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.groups, other.groups))));
  };
  function Argument(chalkTalkTarget) {
    this.chalkTalkTarget = chalkTalkTarget;
  }
  Argument.prototype.forEach_ewgams$ = function (fn) {
    fn(this.chalkTalkTarget);
  };
  Argument.prototype.print_rg88dk$ = function (buffer, level) {
    var tmp$;
    tmp$ = this.chalkTalkTarget;
    if (Kotlin.isType(tmp$, Group))
      this.chalkTalkTarget.print_yrfq27$(buffer, level, true);
    else if (Kotlin.isType(tmp$, ChalkTalkToken)) {
      buffer.append_gw00v9$(AstUtils_getInstance().buildIndent_fzusl$(level, true));
      buffer.append_gw00v9$(this.chalkTalkTarget.text);
      buffer.append_gw00v9$('\n');
    }
     else if (Kotlin.isType(tmp$, Abstraction)) {
      buffer.append_gw00v9$(AstUtils_getInstance().buildIndent_fzusl$(level, true));
      buffer.append_gw00v9$(this.chalkTalkTarget.toCode());
      buffer.append_gw00v9$('\n');
    }
     else if (Kotlin.isType(tmp$, Aggregate)) {
      buffer.append_gw00v9$(AstUtils_getInstance().buildIndent_fzusl$(level, true));
      buffer.append_gw00v9$(this.chalkTalkTarget.toCode());
      buffer.append_gw00v9$('\n');
    }
     else if (Kotlin.isType(tmp$, Assignment)) {
      buffer.append_gw00v9$(AstUtils_getInstance().buildIndent_fzusl$(level, true));
      buffer.append_gw00v9$(this.chalkTalkTarget.toCode());
      buffer.append_gw00v9$('\n');
    }
     else if (Kotlin.isType(tmp$, Mapping)) {
      buffer.append_gw00v9$(AstUtils_getInstance().buildIndent_fzusl$(level, true));
      buffer.append_gw00v9$(this.chalkTalkTarget.toCode());
      buffer.append_gw00v9$('\n');
    }
     else if (Kotlin.isType(tmp$, Tuple)) {
      buffer.append_gw00v9$(AstUtils_getInstance().buildIndent_fzusl$(level, true));
      buffer.append_gw00v9$(this.chalkTalkTarget.toCode());
      buffer.append_gw00v9$('\n');
    }
     else
      Kotlin.noWhenBranchMatched();
  };
  Argument.prototype.toCode = function () {
    var buffer = StringBuilder_init();
    this.print_rg88dk$(buffer, 0);
    return buffer.toString();
  };
  Argument.prototype.resolve = function () {
    return this.chalkTalkTarget.resolve();
  };
  Argument.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Argument',
    interfaces: [ChalkTalkNode]
  };
  Argument.prototype.component1 = function () {
    return this.chalkTalkTarget;
  };
  Argument.prototype.copy_boyym2$ = function (chalkTalkTarget) {
    return new Argument(chalkTalkTarget === void 0 ? this.chalkTalkTarget : chalkTalkTarget);
  };
  Argument.prototype.toString = function () {
    return 'Argument(chalkTalkTarget=' + Kotlin.toString(this.chalkTalkTarget) + ')';
  };
  Argument.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.chalkTalkTarget) | 0;
    return result;
  };
  Argument.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.chalkTalkTarget, other.chalkTalkTarget))));
  };
  function Section(name, args) {
    this.name = name;
    this.args = args;
  }
  Section.prototype.forEach_ewgams$ = function (fn) {
    fn(this.name);
    var tmp$;
    tmp$ = this.args.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  Section.prototype.print_yrfq27$ = function (buffer, level, fromArg) {
    var tmp$;
    buffer.append_gw00v9$(AstUtils_getInstance().buildIndent_fzusl$(level, fromArg));
    buffer.append_gw00v9$(this.name.text);
    buffer.append_gw00v9$(':\n');
    tmp$ = this.args.iterator();
    while (tmp$.hasNext()) {
      var arg = tmp$.next();
      arg.print_rg88dk$(buffer, level + 1 | 0);
    }
  };
  Section.prototype.toCode = function () {
    var buffer = StringBuilder_init();
    this.print_yrfq27$(buffer, 0, false);
    return buffer.toString();
  };
  Section.prototype.resolve = function () {
    return this;
  };
  Section.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Section',
    interfaces: [ChalkTalkNode]
  };
  Section.prototype.component1 = function () {
    return this.name;
  };
  Section.prototype.component2 = function () {
    return this.args;
  };
  Section.prototype.copy_lk98w6$ = function (name, args) {
    return new Section(name === void 0 ? this.name : name, args === void 0 ? this.args : args);
  };
  Section.prototype.toString = function () {
    return 'Section(name=' + Kotlin.toString(this.name) + (', args=' + Kotlin.toString(this.args)) + ')';
  };
  Section.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.name) | 0;
    result = result * 31 + Kotlin.hashCode(this.args) | 0;
    return result;
  };
  Section.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.name, other.name) && Kotlin.equals(this.args, other.args)))));
  };
  function AstUtils() {
    AstUtils_instance = this;
  }
  AstUtils.prototype.max_0 = function (val1, val2) {
    return val1 >= val2 ? val1 : val2;
  };
  AstUtils.prototype.buildIndent_fzusl$ = function (level, isArg) {
    var buffer = StringBuilder_init();
    var numSpaces = isArg ? 2 * this.max_0(level - 1 | 0, 0) | 0 : 2 * level | 0;
    for (var i = 0; i < numSpaces; i++) {
      buffer.append_s8itvh$(32);
    }
    if (isArg) {
      buffer.append_gw00v9$('. ');
    }
    return buffer.toString();
  };
  AstUtils.prototype.getIndent_za3lpa$ = function (size) {
    var buffer = StringBuilder_init();
    for (var i = 0; i < size; i++) {
      buffer.append_s8itvh$(32);
    }
    return buffer.toString();
  };
  function AstUtils$getRow$lambda(closure$rowResult, this$AstUtils) {
    return function (it) {
      if (closure$rowResult.v === -1) {
        var row = this$AstUtils.getRow_rk66c5$(it);
        if (row >= 0) {
          closure$rowResult.v = row;
        }
      }
      return Unit;
    };
  }
  AstUtils.prototype.getRow_rk66c5$ = function (node) {
    if (Kotlin.isType(node, ChalkTalkToken)) {
      return node.row;
    }
    var rowResult = {v: -1};
    node.forEach_ewgams$(AstUtils$getRow$lambda(rowResult, this));
    return rowResult.v;
  };
  function AstUtils$getColumn$lambda(closure$colResult, this$AstUtils) {
    return function (it) {
      if (closure$colResult.v === -1) {
        var col = this$AstUtils.getColumn_rk66c5$(it);
        if (col >= 0) {
          closure$colResult.v = col;
        }
      }
      return Unit;
    };
  }
  AstUtils.prototype.getColumn_rk66c5$ = function (node) {
    if (Kotlin.isType(node, ChalkTalkToken)) {
      return node.column;
    }
    var colResult = {v: -1};
    node.forEach_ewgams$(AstUtils$getColumn$lambda(colResult, this));
    return colResult.v;
  };
  AstUtils.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'AstUtils',
    interfaces: []
  };
  var AstUtils_instance = null;
  function AstUtils_getInstance() {
    if (AstUtils_instance === null) {
      new AstUtils();
    }
    return AstUtils_instance;
  }
  function ChalkTalkTokenType(name, ordinal) {
    Enum.call(this);
    this.name$ = name;
    this.ordinal$ = ordinal;
  }
  function ChalkTalkTokenType_initFields() {
    ChalkTalkTokenType_initFields = function () {
    };
    ChalkTalkTokenType$DotSpace_instance = new ChalkTalkTokenType('DotSpace', 0);
    ChalkTalkTokenType$Name_instance = new ChalkTalkTokenType('Name', 1);
    ChalkTalkTokenType$Colon_instance = new ChalkTalkTokenType('Colon', 2);
    ChalkTalkTokenType$String_instance = new ChalkTalkTokenType('String', 3);
    ChalkTalkTokenType$Statement_instance = new ChalkTalkTokenType('Statement', 4);
    ChalkTalkTokenType$Id_instance = new ChalkTalkTokenType('Id', 5);
    ChalkTalkTokenType$Comma_instance = new ChalkTalkTokenType('Comma', 6);
    ChalkTalkTokenType$Begin_instance = new ChalkTalkTokenType('Begin', 7);
    ChalkTalkTokenType$End_instance = new ChalkTalkTokenType('End', 8);
    ChalkTalkTokenType$Linebreak_instance = new ChalkTalkTokenType('Linebreak', 9);
    ChalkTalkTokenType$Invalid_instance = new ChalkTalkTokenType('Invalid', 10);
    ChalkTalkTokenType$Equals_instance = new ChalkTalkTokenType('Equals', 11);
    ChalkTalkTokenType$ColonEquals_instance = new ChalkTalkTokenType('ColonEquals', 12);
    ChalkTalkTokenType$LParen_instance = new ChalkTalkTokenType('LParen', 13);
    ChalkTalkTokenType$RParen_instance = new ChalkTalkTokenType('RParen', 14);
    ChalkTalkTokenType$LCurly_instance = new ChalkTalkTokenType('LCurly', 15);
    ChalkTalkTokenType$RCurly_instance = new ChalkTalkTokenType('RCurly', 16);
  }
  var ChalkTalkTokenType$DotSpace_instance;
  function ChalkTalkTokenType$DotSpace_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$DotSpace_instance;
  }
  var ChalkTalkTokenType$Name_instance;
  function ChalkTalkTokenType$Name_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$Name_instance;
  }
  var ChalkTalkTokenType$Colon_instance;
  function ChalkTalkTokenType$Colon_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$Colon_instance;
  }
  var ChalkTalkTokenType$String_instance;
  function ChalkTalkTokenType$String_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$String_instance;
  }
  var ChalkTalkTokenType$Statement_instance;
  function ChalkTalkTokenType$Statement_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$Statement_instance;
  }
  var ChalkTalkTokenType$Id_instance;
  function ChalkTalkTokenType$Id_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$Id_instance;
  }
  var ChalkTalkTokenType$Comma_instance;
  function ChalkTalkTokenType$Comma_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$Comma_instance;
  }
  var ChalkTalkTokenType$Begin_instance;
  function ChalkTalkTokenType$Begin_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$Begin_instance;
  }
  var ChalkTalkTokenType$End_instance;
  function ChalkTalkTokenType$End_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$End_instance;
  }
  var ChalkTalkTokenType$Linebreak_instance;
  function ChalkTalkTokenType$Linebreak_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$Linebreak_instance;
  }
  var ChalkTalkTokenType$Invalid_instance;
  function ChalkTalkTokenType$Invalid_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$Invalid_instance;
  }
  var ChalkTalkTokenType$Equals_instance;
  function ChalkTalkTokenType$Equals_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$Equals_instance;
  }
  var ChalkTalkTokenType$ColonEquals_instance;
  function ChalkTalkTokenType$ColonEquals_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$ColonEquals_instance;
  }
  var ChalkTalkTokenType$LParen_instance;
  function ChalkTalkTokenType$LParen_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$LParen_instance;
  }
  var ChalkTalkTokenType$RParen_instance;
  function ChalkTalkTokenType$RParen_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$RParen_instance;
  }
  var ChalkTalkTokenType$LCurly_instance;
  function ChalkTalkTokenType$LCurly_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$LCurly_instance;
  }
  var ChalkTalkTokenType$RCurly_instance;
  function ChalkTalkTokenType$RCurly_getInstance() {
    ChalkTalkTokenType_initFields();
    return ChalkTalkTokenType$RCurly_instance;
  }
  ChalkTalkTokenType.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ChalkTalkTokenType',
    interfaces: [Enum]
  };
  function ChalkTalkTokenType$values() {
    return [ChalkTalkTokenType$DotSpace_getInstance(), ChalkTalkTokenType$Name_getInstance(), ChalkTalkTokenType$Colon_getInstance(), ChalkTalkTokenType$String_getInstance(), ChalkTalkTokenType$Statement_getInstance(), ChalkTalkTokenType$Id_getInstance(), ChalkTalkTokenType$Comma_getInstance(), ChalkTalkTokenType$Begin_getInstance(), ChalkTalkTokenType$End_getInstance(), ChalkTalkTokenType$Linebreak_getInstance(), ChalkTalkTokenType$Invalid_getInstance(), ChalkTalkTokenType$Equals_getInstance(), ChalkTalkTokenType$ColonEquals_getInstance(), ChalkTalkTokenType$LParen_getInstance(), ChalkTalkTokenType$RParen_getInstance(), ChalkTalkTokenType$LCurly_getInstance(), ChalkTalkTokenType$RCurly_getInstance()];
  }
  ChalkTalkTokenType.values = ChalkTalkTokenType$values;
  function ChalkTalkTokenType$valueOf(name) {
    switch (name) {
      case 'DotSpace':
        return ChalkTalkTokenType$DotSpace_getInstance();
      case 'Name':
        return ChalkTalkTokenType$Name_getInstance();
      case 'Colon':
        return ChalkTalkTokenType$Colon_getInstance();
      case 'String':
        return ChalkTalkTokenType$String_getInstance();
      case 'Statement':
        return ChalkTalkTokenType$Statement_getInstance();
      case 'Id':
        return ChalkTalkTokenType$Id_getInstance();
      case 'Comma':
        return ChalkTalkTokenType$Comma_getInstance();
      case 'Begin':
        return ChalkTalkTokenType$Begin_getInstance();
      case 'End':
        return ChalkTalkTokenType$End_getInstance();
      case 'Linebreak':
        return ChalkTalkTokenType$Linebreak_getInstance();
      case 'Invalid':
        return ChalkTalkTokenType$Invalid_getInstance();
      case 'Equals':
        return ChalkTalkTokenType$Equals_getInstance();
      case 'ColonEquals':
        return ChalkTalkTokenType$ColonEquals_getInstance();
      case 'LParen':
        return ChalkTalkTokenType$LParen_getInstance();
      case 'RParen':
        return ChalkTalkTokenType$RParen_getInstance();
      case 'LCurly':
        return ChalkTalkTokenType$LCurly_getInstance();
      case 'RCurly':
        return ChalkTalkTokenType$RCurly_getInstance();
      default:throwISE('No enum constant mathlingua.common.chalktalk.phase1.ast.ChalkTalkTokenType.' + name);
    }
  }
  ChalkTalkTokenType.valueOf_61zpoe$ = ChalkTalkTokenType$valueOf;
  function ChalkTalkTarget() {
  }
  ChalkTalkTarget.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ChalkTalkTarget',
    interfaces: [ChalkTalkNode]
  };
  function TupleItem() {
    ChalkTalkTarget.call(this);
  }
  TupleItem.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'TupleItem',
    interfaces: [ChalkTalkTarget]
  };
  function AssignmentRhs() {
    TupleItem.call(this);
  }
  AssignmentRhs.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'AssignmentRhs',
    interfaces: [TupleItem]
  };
  function ChalkTalkToken(text, type, row, column) {
    AssignmentRhs.call(this);
    this.text = text;
    this.type = type;
    this.row = row;
    this.column = column;
  }
  ChalkTalkToken.prototype.forEach_ewgams$ = function (fn) {
  };
  ChalkTalkToken.prototype.toCode = function () {
    return this.text;
  };
  ChalkTalkToken.prototype.resolve = function () {
    return this;
  };
  ChalkTalkToken.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ChalkTalkToken',
    interfaces: [AssignmentRhs]
  };
  ChalkTalkToken.prototype.component1 = function () {
    return this.text;
  };
  ChalkTalkToken.prototype.component2 = function () {
    return this.type;
  };
  ChalkTalkToken.prototype.component3 = function () {
    return this.row;
  };
  ChalkTalkToken.prototype.component4 = function () {
    return this.column;
  };
  ChalkTalkToken.prototype.copy_m2738k$ = function (text, type, row, column) {
    return new ChalkTalkToken(text === void 0 ? this.text : text, type === void 0 ? this.type : type, row === void 0 ? this.row : row, column === void 0 ? this.column : column);
  };
  ChalkTalkToken.prototype.toString = function () {
    return 'ChalkTalkToken(text=' + Kotlin.toString(this.text) + (', type=' + Kotlin.toString(this.type)) + (', row=' + Kotlin.toString(this.row)) + (', column=' + Kotlin.toString(this.column)) + ')';
  };
  ChalkTalkToken.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.text) | 0;
    result = result * 31 + Kotlin.hashCode(this.type) | 0;
    result = result * 31 + Kotlin.hashCode(this.row) | 0;
    result = result * 31 + Kotlin.hashCode(this.column) | 0;
    return result;
  };
  ChalkTalkToken.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.text, other.text) && Kotlin.equals(this.type, other.type) && Kotlin.equals(this.row, other.row) && Kotlin.equals(this.column, other.column)))));
  };
  function Mapping(lhs, rhs) {
    ChalkTalkTarget.call(this);
    this.lhs = lhs;
    this.rhs = rhs;
  }
  Mapping.prototype.forEach_ewgams$ = function (fn) {
    fn(this.lhs);
    fn(this.rhs);
  };
  Mapping.prototype.toCode = function () {
    return this.lhs.toCode() + ' = ' + this.rhs.toCode();
  };
  Mapping.prototype.resolve = function () {
    return this;
  };
  Mapping.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Mapping',
    interfaces: [ChalkTalkTarget]
  };
  Mapping.prototype.component1 = function () {
    return this.lhs;
  };
  Mapping.prototype.component2 = function () {
    return this.rhs;
  };
  Mapping.prototype.copy_9wvzv0$ = function (lhs, rhs) {
    return new Mapping(lhs === void 0 ? this.lhs : lhs, rhs === void 0 ? this.rhs : rhs);
  };
  Mapping.prototype.toString = function () {
    return 'Mapping(lhs=' + Kotlin.toString(this.lhs) + (', rhs=' + Kotlin.toString(this.rhs)) + ')';
  };
  Mapping.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.lhs) | 0;
    result = result * 31 + Kotlin.hashCode(this.rhs) | 0;
    return result;
  };
  Mapping.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.lhs, other.lhs) && Kotlin.equals(this.rhs, other.rhs)))));
  };
  function Group(sections, id) {
    ChalkTalkTarget.call(this);
    this.sections = sections;
    this.id = id;
  }
  Group.prototype.forEach_ewgams$ = function (fn) {
    if (this.id != null) {
      fn(this.id);
    }
    var tmp$;
    tmp$ = this.sections.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  Group.prototype.print_yrfq27$ = function (buffer, level, fromArg) {
    var tmp$;
    if (this.id != null) {
      buffer.append_gw00v9$(this.id.text);
      buffer.append_gw00v9$('\n');
    }
    var first = true;
    tmp$ = this.sections.iterator();
    while (tmp$.hasNext()) {
      var sect = tmp$.next();
      if (first) {
        sect.print_yrfq27$(buffer, level, fromArg && this.id == null);
      }
       else {
        sect.print_yrfq27$(buffer, level, false);
      }
      first = false;
    }
  };
  Group.prototype.toCode = function () {
    var buffer = StringBuilder_init();
    this.print_yrfq27$(buffer, 0, false);
    return buffer.toString();
  };
  Group.prototype.resolve = function () {
    return this;
  };
  Group.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Group',
    interfaces: [ChalkTalkTarget]
  };
  Group.prototype.component1 = function () {
    return this.sections;
  };
  Group.prototype.component2 = function () {
    return this.id;
  };
  Group.prototype.copy_5yv6a1$ = function (sections, id) {
    return new Group(sections === void 0 ? this.sections : sections, id === void 0 ? this.id : id);
  };
  Group.prototype.toString = function () {
    return 'Group(sections=' + Kotlin.toString(this.sections) + (', id=' + Kotlin.toString(this.id)) + ')';
  };
  Group.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.sections) | 0;
    result = result * 31 + Kotlin.hashCode(this.id) | 0;
    return result;
  };
  Group.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.sections, other.sections) && Kotlin.equals(this.id, other.id)))));
  };
  function Assignment(lhs, rhs) {
    TupleItem.call(this);
    this.lhs = lhs;
    this.rhs = rhs;
  }
  Assignment.prototype.forEach_ewgams$ = function (fn) {
    fn(this.lhs);
    fn(this.rhs);
  };
  Assignment.prototype.toCode = function () {
    return this.lhs.toCode() + ' := ' + this.rhs.toCode();
  };
  Assignment.prototype.resolve = function () {
    return this;
  };
  Assignment.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Assignment',
    interfaces: [TupleItem]
  };
  Assignment.prototype.component1 = function () {
    return this.lhs;
  };
  Assignment.prototype.component2 = function () {
    return this.rhs;
  };
  Assignment.prototype.copy_k8hnm6$ = function (lhs, rhs) {
    return new Assignment(lhs === void 0 ? this.lhs : lhs, rhs === void 0 ? this.rhs : rhs);
  };
  Assignment.prototype.toString = function () {
    return 'Assignment(lhs=' + Kotlin.toString(this.lhs) + (', rhs=' + Kotlin.toString(this.rhs)) + ')';
  };
  Assignment.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.lhs) | 0;
    result = result * 31 + Kotlin.hashCode(this.rhs) | 0;
    return result;
  };
  Assignment.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.lhs, other.lhs) && Kotlin.equals(this.rhs, other.rhs)))));
  };
  function Tuple(items) {
    AssignmentRhs.call(this);
    this.items = items;
  }
  Tuple.prototype.forEach_ewgams$ = function (fn) {
    var tmp$;
    tmp$ = this.items.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  Tuple.prototype.toCode = function () {
    var tmp$;
    var builder = StringBuilder_init();
    builder.append_s8itvh$(40);
    tmp$ = this.items.size;
    for (var i = 0; i < tmp$; i++) {
      builder.append_gw00v9$(this.items.get_za3lpa$(i).toCode());
      if (i !== (this.items.size - 1 | 0)) {
        builder.append_gw00v9$(', ');
      }
    }
    builder.append_s8itvh$(41);
    return builder.toString();
  };
  Tuple.prototype.resolve = function () {
    return this;
  };
  Tuple.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Tuple',
    interfaces: [AssignmentRhs]
  };
  Tuple.prototype.component1 = function () {
    return this.items;
  };
  Tuple.prototype.copy_w950eq$ = function (items) {
    return new Tuple(items === void 0 ? this.items : items);
  };
  Tuple.prototype.toString = function () {
    return 'Tuple(items=' + Kotlin.toString(this.items) + ')';
  };
  Tuple.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.items) | 0;
    return result;
  };
  Tuple.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.items, other.items))));
  };
  function Abstraction(name, params) {
    TupleItem.call(this);
    this.name = name;
    this.params = params;
  }
  Abstraction.prototype.forEach_ewgams$ = function (fn) {
    fn(this.name);
    var tmp$;
    tmp$ = this.params.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  Abstraction.prototype.toCode = function () {
    var tmp$;
    var builder = StringBuilder_init();
    builder.append_gw00v9$(this.name.toCode());
    builder.append_s8itvh$(40);
    tmp$ = this.params.size;
    for (var i = 0; i < tmp$; i++) {
      builder.append_gw00v9$(this.params.get_za3lpa$(i).toCode());
      if (i !== (this.params.size - 1 | 0)) {
        builder.append_gw00v9$(', ');
      }
    }
    builder.append_s8itvh$(41);
    return builder.toString();
  };
  Abstraction.prototype.resolve = function () {
    return this;
  };
  Abstraction.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Abstraction',
    interfaces: [TupleItem]
  };
  Abstraction.prototype.component1 = function () {
    return this.name;
  };
  Abstraction.prototype.component2 = function () {
    return this.params;
  };
  Abstraction.prototype.copy_smjr0l$ = function (name, params) {
    return new Abstraction(name === void 0 ? this.name : name, params === void 0 ? this.params : params);
  };
  Abstraction.prototype.toString = function () {
    return 'Abstraction(name=' + Kotlin.toString(this.name) + (', params=' + Kotlin.toString(this.params)) + ')';
  };
  Abstraction.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.name) | 0;
    result = result * 31 + Kotlin.hashCode(this.params) | 0;
    return result;
  };
  Abstraction.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.name, other.name) && Kotlin.equals(this.params, other.params)))));
  };
  function Aggregate(params) {
    AssignmentRhs.call(this);
    this.params = params;
  }
  Aggregate.prototype.forEach_ewgams$ = function (fn) {
    var tmp$;
    tmp$ = this.params.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  Aggregate.prototype.toCode = function () {
    var tmp$;
    var builder = StringBuilder_init();
    builder.append_s8itvh$(123);
    tmp$ = this.params.size;
    for (var i = 0; i < tmp$; i++) {
      builder.append_gw00v9$(this.params.get_za3lpa$(i).toCode());
      if (i !== (this.params.size - 1 | 0)) {
        builder.append_gw00v9$(', ');
      }
    }
    builder.append_s8itvh$(125);
    return builder.toString();
  };
  Aggregate.prototype.resolve = function () {
    return this;
  };
  Aggregate.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Aggregate',
    interfaces: [AssignmentRhs]
  };
  Aggregate.prototype.component1 = function () {
    return this.params;
  };
  Aggregate.prototype.copy_k63fjz$ = function (params) {
    return new Aggregate(params === void 0 ? this.params : params);
  };
  Aggregate.prototype.toString = function () {
    return 'Aggregate(params=' + Kotlin.toString(this.params) + ')';
  };
  Aggregate.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.params) | 0;
    return result;
  };
  Aggregate.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.params, other.params))));
  };
  function AliasSection(mappings) {
    AliasSection$Companion_getInstance();
    this.mappings = mappings;
  }
  AliasSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.mappings.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  AliasSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var tmp$;
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'Alias:'));
    builder.append_s8itvh$(10);
    tmp$ = this.mappings.size;
    for (var i = 0; i < tmp$; i++) {
      builder.append_gw00v9$(this.mappings.get_za3lpa$(i).toCode_eltk6l$(true, indent + 2 | 0));
      if (i !== (this.mappings.size - 1 | 0)) {
        builder.append_s8itvh$(10);
      }
    }
    return builder.toString();
  };
  function AliasSection$Companion() {
    AliasSection$Companion_instance = this;
  }
  AliasSection$Companion.prototype.validate_3fjnpj$ = function (section) {
    var tmp$, tmp$_0;
    if (!equals(section.name.text, 'Alias')) {
      return Validation$Companion_getInstance().failure_rg4ulb$(listOf(new ParseError("Expected a 'Alias' but found '" + section.name.text + "'", AstUtils_getInstance().getRow_rk66c5$(section), AstUtils_getInstance().getColumn_rk66c5$(section))));
    }
    var errors = ArrayList_init();
    var mappings = ArrayList_init();
    tmp$ = section.args.iterator();
    while (tmp$.hasNext()) {
      var arg = tmp$.next();
      var validation = MappingNode$Companion_getInstance().validate_rk66c5$(arg);
      if (validation.isSuccessful) {
        mappings.add_11rb$(ensureNotNull(validation.value));
      }
       else {
        errors.addAll_brywnq$(validation.errors);
      }
    }
    if (!errors.isEmpty()) {
      tmp$_0 = Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
     else {
      tmp$_0 = Validation$Companion_getInstance().success_mh5how$(new AliasSection(mappings));
    }
    return tmp$_0;
  };
  AliasSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var AliasSection$Companion_instance = null;
  function AliasSection$Companion_getInstance() {
    if (AliasSection$Companion_instance === null) {
      new AliasSection$Companion();
    }
    return AliasSection$Companion_instance;
  }
  AliasSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'AliasSection',
    interfaces: [Phase2Node]
  };
  AliasSection.prototype.component1 = function () {
    return this.mappings;
  };
  AliasSection.prototype.copy_rz3npo$ = function (mappings) {
    return new AliasSection(mappings === void 0 ? this.mappings : mappings);
  };
  AliasSection.prototype.toString = function () {
    return 'AliasSection(mappings=' + Kotlin.toString(this.mappings) + ')';
  };
  AliasSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.mappings) | 0;
    return result;
  };
  AliasSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.mappings, other.mappings))));
  };
  function indentedString(useDot, indent, line) {
    var tmp$;
    var builder = StringBuilder_init();
    tmp$ = indent - 2 | 0;
    for (var i = 0; i < tmp$; i++) {
      builder.append_s8itvh$(32);
    }
    if ((indent - 2 | 0) >= 0) {
      builder.append_s8itvh$(useDot ? 46 : 32);
    }
    if ((indent - 1 | 0) >= 0) {
      builder.append_s8itvh$(32);
    }
    builder.append_gw00v9$(line);
    return builder.toString();
  }
  function ValidationPair(matches, validate) {
    this.matches = matches;
    this.validate = validate;
  }
  ValidationPair.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ValidationPair',
    interfaces: []
  };
  ValidationPair.prototype.component1 = function () {
    return this.matches;
  };
  ValidationPair.prototype.component2 = function () {
    return this.validate;
  };
  ValidationPair.prototype.copy_ji3ymy$ = function (matches, validate) {
    return new ValidationPair(matches === void 0 ? this.matches : matches, validate === void 0 ? this.validate : validate);
  };
  ValidationPair.prototype.toString = function () {
    return 'ValidationPair(matches=' + Kotlin.toString(this.matches) + (', validate=' + Kotlin.toString(this.validate)) + ')';
  };
  ValidationPair.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.matches) | 0;
    result = result * 31 + Kotlin.hashCode(this.validate) | 0;
    return result;
  };
  ValidationPair.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.matches, other.matches) && Kotlin.equals(this.validate, other.validate)))));
  };
  var CLAUSE_VALIDATORS;
  function Clause() {
    Clause$Companion_getInstance();
  }
  function Clause$Companion() {
    Clause$Companion_instance = this;
  }
  Clause$Companion.prototype.validate_rk66c5$ = function (rawNode) {
    var tmp$, tmp$_0;
    var node = rawNode.resolve();
    tmp$ = CLAUSE_VALIDATORS.iterator();
    while (tmp$.hasNext()) {
      var pair = tmp$.next();
      if (pair.matches(node)) {
        var validation = pair.validate(node);
        if (validation.isSuccessful) {
          return Validation$Companion_getInstance().success_mh5how$(ensureNotNull(validation.value));
        }
         else {
          tmp$_0 = Validation$Companion_getInstance().failure_rg4ulb$(validation.errors);
        }
        return tmp$_0;
      }
    }
    return Validation$Companion_getInstance().failure_rg4ulb$(listOf(new ParseError('Expected a Target', AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node))));
  };
  Clause$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var Clause$Companion_instance = null;
  function Clause$Companion_getInstance() {
    if (Clause$Companion_instance === null) {
      new Clause$Companion();
    }
    return Clause$Companion_instance;
  }
  Clause.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Clause',
    interfaces: [Phase2Node]
  };
  function Target() {
    Clause.call(this);
  }
  Target.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Target',
    interfaces: [Clause]
  };
  function AbstractionNode(abstraction) {
    AbstractionNode$Companion_getInstance();
    Target.call(this);
    this.abstraction = abstraction;
  }
  AbstractionNode.prototype.forEach_ye21ev$ = function (fn) {
  };
  AbstractionNode.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode(isArg, indent, this.abstraction);
  };
  function AbstractionNode$Companion() {
    AbstractionNode$Companion_instance = this;
  }
  AbstractionNode$Companion.prototype.isAbstraction_rk66c5$ = function (node) {
    return Kotlin.isType(node, Abstraction);
  };
  function AbstractionNode$Companion$validate$lambda(it) {
    var tmp$;
    return Kotlin.isType(tmp$ = it, Abstraction) ? tmp$ : null;
  }
  function AbstractionNode$Companion$validate$lambda_0(it) {
    return new AbstractionNode(it);
  }
  AbstractionNode$Companion.prototype.validate_rk66c5$ = function (node) {
    return validateWrappedNode(node, 'AbstractionNode', AbstractionNode$Companion$validate$lambda, AbstractionNode$Companion$validate$lambda_0);
  };
  AbstractionNode$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var AbstractionNode$Companion_instance = null;
  function AbstractionNode$Companion_getInstance() {
    if (AbstractionNode$Companion_instance === null) {
      new AbstractionNode$Companion();
    }
    return AbstractionNode$Companion_instance;
  }
  AbstractionNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'AbstractionNode',
    interfaces: [Target]
  };
  AbstractionNode.prototype.component1 = function () {
    return this.abstraction;
  };
  AbstractionNode.prototype.copy_mnh2mg$ = function (abstraction) {
    return new AbstractionNode(abstraction === void 0 ? this.abstraction : abstraction);
  };
  AbstractionNode.prototype.toString = function () {
    return 'AbstractionNode(abstraction=' + Kotlin.toString(this.abstraction) + ')';
  };
  AbstractionNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.abstraction) | 0;
    return result;
  };
  AbstractionNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.abstraction, other.abstraction))));
  };
  function AggregateNode(aggregate) {
    AggregateNode$Companion_getInstance();
    Target.call(this);
    this.aggregate = aggregate;
  }
  AggregateNode.prototype.forEach_ye21ev$ = function (fn) {
  };
  AggregateNode.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode(isArg, indent, this.aggregate);
  };
  function AggregateNode$Companion() {
    AggregateNode$Companion_instance = this;
  }
  AggregateNode$Companion.prototype.isAggregate_rk66c5$ = function (node) {
    return Kotlin.isType(node, Aggregate);
  };
  function AggregateNode$Companion$validate$lambda(it) {
    var tmp$;
    return Kotlin.isType(tmp$ = it, Aggregate) ? tmp$ : null;
  }
  function AggregateNode$Companion$validate$lambda_0(it) {
    return new AggregateNode(it);
  }
  AggregateNode$Companion.prototype.validate_rk66c5$ = function (node) {
    return validateWrappedNode(node, 'AggregateNode', AggregateNode$Companion$validate$lambda, AggregateNode$Companion$validate$lambda_0);
  };
  AggregateNode$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var AggregateNode$Companion_instance = null;
  function AggregateNode$Companion_getInstance() {
    if (AggregateNode$Companion_instance === null) {
      new AggregateNode$Companion();
    }
    return AggregateNode$Companion_instance;
  }
  AggregateNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'AggregateNode',
    interfaces: [Target]
  };
  AggregateNode.prototype.component1 = function () {
    return this.aggregate;
  };
  AggregateNode.prototype.copy_qc4esf$ = function (aggregate) {
    return new AggregateNode(aggregate === void 0 ? this.aggregate : aggregate);
  };
  AggregateNode.prototype.toString = function () {
    return 'AggregateNode(aggregate=' + Kotlin.toString(this.aggregate) + ')';
  };
  AggregateNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.aggregate) | 0;
    return result;
  };
  AggregateNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.aggregate, other.aggregate))));
  };
  function TupleNode(tuple) {
    TupleNode$Companion_getInstance();
    Target.call(this);
    this.tuple = tuple;
  }
  TupleNode.prototype.forEach_ye21ev$ = function (fn) {
  };
  TupleNode.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode(isArg, indent, this.tuple);
  };
  function TupleNode$Companion() {
    TupleNode$Companion_instance = this;
  }
  TupleNode$Companion.prototype.isTuple_rk66c5$ = function (node) {
    return Kotlin.isType(node, Tuple);
  };
  function TupleNode$Companion$validate$lambda(it) {
    var tmp$;
    return Kotlin.isType(tmp$ = it, Tuple) ? tmp$ : null;
  }
  function TupleNode$Companion$validate$lambda_0(it) {
    return new TupleNode(it);
  }
  TupleNode$Companion.prototype.validate_rk66c5$ = function (node) {
    return validateWrappedNode(node, 'TupleNode', TupleNode$Companion$validate$lambda, TupleNode$Companion$validate$lambda_0);
  };
  TupleNode$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var TupleNode$Companion_instance = null;
  function TupleNode$Companion_getInstance() {
    if (TupleNode$Companion_instance === null) {
      new TupleNode$Companion();
    }
    return TupleNode$Companion_instance;
  }
  TupleNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'TupleNode',
    interfaces: [Target]
  };
  TupleNode.prototype.component1 = function () {
    return this.tuple;
  };
  TupleNode.prototype.copy_hsayfq$ = function (tuple) {
    return new TupleNode(tuple === void 0 ? this.tuple : tuple);
  };
  TupleNode.prototype.toString = function () {
    return 'TupleNode(tuple=' + Kotlin.toString(this.tuple) + ')';
  };
  TupleNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.tuple) | 0;
    return result;
  };
  TupleNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.tuple, other.tuple))));
  };
  function AssignmentNode(assignment) {
    AssignmentNode$Companion_getInstance();
    Target.call(this);
    this.assignment = assignment;
  }
  AssignmentNode.prototype.forEach_ye21ev$ = function (fn) {
  };
  AssignmentNode.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode(isArg, indent, this.assignment);
  };
  function AssignmentNode$Companion() {
    AssignmentNode$Companion_instance = this;
  }
  AssignmentNode$Companion.prototype.isAssignment_rk66c5$ = function (node) {
    return Kotlin.isType(node, Assignment);
  };
  function AssignmentNode$Companion$validate$lambda(it) {
    var tmp$;
    return Kotlin.isType(tmp$ = it, Assignment) ? tmp$ : null;
  }
  AssignmentNode$Companion.prototype.validate_rk66c5$ = function (node) {
    return validateWrappedNode(node, 'AssignmentNode', AssignmentNode$Companion$validate$lambda, getCallableRef('AssignmentNode', function (assignment) {
      return new AssignmentNode(assignment);
    }));
  };
  AssignmentNode$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var AssignmentNode$Companion_instance = null;
  function AssignmentNode$Companion_getInstance() {
    if (AssignmentNode$Companion_instance === null) {
      new AssignmentNode$Companion();
    }
    return AssignmentNode$Companion_instance;
  }
  AssignmentNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'AssignmentNode',
    interfaces: [Target]
  };
  AssignmentNode.prototype.component1 = function () {
    return this.assignment;
  };
  AssignmentNode.prototype.copy_y4i2ej$ = function (assignment) {
    return new AssignmentNode(assignment === void 0 ? this.assignment : assignment);
  };
  AssignmentNode.prototype.toString = function () {
    return 'AssignmentNode(assignment=' + Kotlin.toString(this.assignment) + ')';
  };
  AssignmentNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.assignment) | 0;
    return result;
  };
  AssignmentNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.assignment, other.assignment))));
  };
  function MappingNode(mapping) {
    MappingNode$Companion_getInstance();
    this.mapping = mapping;
  }
  MappingNode.prototype.forEach_ye21ev$ = function (fn) {
  };
  MappingNode.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode(isArg, indent, this.mapping);
  };
  function MappingNode$Companion() {
    MappingNode$Companion_instance = this;
  }
  MappingNode$Companion.prototype.isMapping_rk66c5$ = function (node) {
    return Kotlin.isType(node, Mapping);
  };
  function MappingNode$Companion$validate$lambda(it) {
    var tmp$;
    return Kotlin.isType(tmp$ = it, Mapping) ? tmp$ : null;
  }
  MappingNode$Companion.prototype.validate_rk66c5$ = function (node) {
    return validateWrappedNode(node, 'MappingNode', MappingNode$Companion$validate$lambda, getCallableRef('MappingNode', function (mapping) {
      return new MappingNode(mapping);
    }));
  };
  MappingNode$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var MappingNode$Companion_instance = null;
  function MappingNode$Companion_getInstance() {
    if (MappingNode$Companion_instance === null) {
      new MappingNode$Companion();
    }
    return MappingNode$Companion_instance;
  }
  MappingNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'MappingNode',
    interfaces: [Phase2Node]
  };
  MappingNode.prototype.component1 = function () {
    return this.mapping;
  };
  MappingNode.prototype.copy_fatphs$ = function (mapping) {
    return new MappingNode(mapping === void 0 ? this.mapping : mapping);
  };
  MappingNode.prototype.toString = function () {
    return 'MappingNode(mapping=' + Kotlin.toString(this.mapping) + ')';
  };
  MappingNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.mapping) | 0;
    return result;
  };
  MappingNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.mapping, other.mapping))));
  };
  function Identifier(name) {
    Identifier$Companion_getInstance();
    Target.call(this);
    this.name = name;
  }
  Identifier.prototype.forEach_ye21ev$ = function (fn) {
  };
  Identifier.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return indentedString(isArg, indent, this.name);
  };
  function Identifier$Companion() {
    Identifier$Companion_instance = this;
  }
  Identifier$Companion.prototype.isIdentifier_rk66c5$ = function (node) {
    return Kotlin.isType(node, ChalkTalkToken) && node.type === ChalkTalkTokenType$Name_getInstance();
  };
  Identifier$Companion.prototype.validate_rk66c5$ = function (rawNode) {
    var node = rawNode.resolve();
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, ChalkTalkToken)) {
      errors.add_11rb$(new ParseError('Cannot convert to a ChalkTalkToken', AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
      return Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
    var text = node.component1()
    , type = node.component2()
    , row = node.component3()
    , column = node.component4();
    if (type !== ChalkTalkTokenType$Name_getInstance()) {
      errors.add_11rb$(new ParseError('A token of type ' + type + ' is not an identifier', row, column));
      return Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
    return Validation$Companion_getInstance().success_mh5how$(new Identifier(text));
  };
  Identifier$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var Identifier$Companion_instance = null;
  function Identifier$Companion_getInstance() {
    if (Identifier$Companion_instance === null) {
      new Identifier$Companion();
    }
    return Identifier$Companion_instance;
  }
  Identifier.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Identifier',
    interfaces: [Target]
  };
  Identifier.prototype.component1 = function () {
    return this.name;
  };
  Identifier.prototype.copy_61zpoe$ = function (name) {
    return new Identifier(name === void 0 ? this.name : name);
  };
  Identifier.prototype.toString = function () {
    return 'Identifier(name=' + Kotlin.toString(this.name) + ')';
  };
  Identifier.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.name) | 0;
    return result;
  };
  Identifier.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.name, other.name))));
  };
  function Statement(text, texTalkRoot) {
    Statement$Companion_getInstance();
    Clause.call(this);
    this.text = text;
    this.texTalkRoot = texTalkRoot;
  }
  Statement.prototype.forEach_ye21ev$ = function (fn) {
  };
  Statement.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return indentedString(isArg, indent, "'" + this.text + "'");
  };
  function Statement$Companion() {
    Statement$Companion_instance = this;
  }
  Statement$Companion.prototype.isStatement_rk66c5$ = function (node) {
    return Kotlin.isType(node, ChalkTalkToken) && node.type === ChalkTalkTokenType$Statement_getInstance();
  };
  Statement$Companion.prototype.validate_rk66c5$ = function (rawNode) {
    var tmp$;
    var node = rawNode.resolve();
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, ChalkTalkToken)) {
      errors.add_11rb$(new ParseError('Cannot convert a to a ChalkTalkToken', AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
    }
    var tmp$_0 = Kotlin.isType(tmp$ = node, ChalkTalkToken) ? tmp$ : throwCCE();
    var rawText = tmp$_0.component1()
    , type = tmp$_0.component2()
    , row = tmp$_0.component3()
    , column = tmp$_0.component4();
    if (type !== ChalkTalkTokenType$Statement_getInstance()) {
      errors.add_11rb$(new ParseError('Cannot convert a ' + node.toCode() + ' to a Statement', row, column));
      return Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
    var endIndex = rawText.length - 1 | 0;
    var text = rawText.substring(1, endIndex);
    var texTalkErrors = ArrayList_init();
    var lexer = newTexTalkLexer(text);
    texTalkErrors.addAll_brywnq$(lexer.errors);
    var parser = newTexTalkParser();
    var result = parser.parse_2mg13h$(lexer);
    texTalkErrors.addAll_brywnq$(result.errors);
    var validation = texTalkErrors.isEmpty() ? Validation$Companion_getInstance().success_mh5how$(result.root) : Validation$Companion_getInstance().failure_rg4ulb$(texTalkErrors);
    return Validation$Companion_getInstance().success_mh5how$(new Statement(text, validation));
  };
  Statement$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var Statement$Companion_instance = null;
  function Statement$Companion_getInstance() {
    if (Statement$Companion_instance === null) {
      new Statement$Companion();
    }
    return Statement$Companion_instance;
  }
  Statement.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Statement',
    interfaces: [Clause]
  };
  Statement.prototype.component1 = function () {
    return this.text;
  };
  Statement.prototype.component2 = function () {
    return this.texTalkRoot;
  };
  Statement.prototype.copy_fe7ugk$ = function (text, texTalkRoot) {
    return new Statement(text === void 0 ? this.text : text, texTalkRoot === void 0 ? this.texTalkRoot : texTalkRoot);
  };
  Statement.prototype.toString = function () {
    return 'Statement(text=' + Kotlin.toString(this.text) + (', texTalkRoot=' + Kotlin.toString(this.texTalkRoot)) + ')';
  };
  Statement.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.text) | 0;
    result = result * 31 + Kotlin.hashCode(this.texTalkRoot) | 0;
    return result;
  };
  Statement.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.text, other.text) && Kotlin.equals(this.texTalkRoot, other.texTalkRoot)))));
  };
  function Text(text) {
    Text$Companion_getInstance();
    Clause.call(this);
    this.text = text;
  }
  Text.prototype.forEach_ye21ev$ = function (fn) {
  };
  Text.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return indentedString(isArg, indent, this.text);
  };
  function Text$Companion() {
    Text$Companion_instance = this;
  }
  Text$Companion.prototype.isText_rk66c5$ = function (node) {
    return Kotlin.isType(node, ChalkTalkToken) && node.type === ChalkTalkTokenType$String_getInstance();
  };
  Text$Companion.prototype.validate_rk66c5$ = function (rawNode) {
    var tmp$;
    var node = rawNode.resolve();
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, ChalkTalkToken)) {
      errors.add_11rb$(new ParseError('Cannot convert a to a ChalkTalkToken', AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
    }
    var tmp$_0 = Kotlin.isType(tmp$ = node, ChalkTalkToken) ? tmp$ : throwCCE();
    var text = tmp$_0.component1()
    , type = tmp$_0.component2()
    , row = tmp$_0.component3()
    , column = tmp$_0.component4();
    if (type !== ChalkTalkTokenType$String_getInstance()) {
      errors.add_11rb$(new ParseError('Cannot convert a ' + node.toCode() + ' to Text', row, column));
      return Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
    return Validation$Companion_getInstance().success_mh5how$(new Text(text));
  };
  Text$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var Text$Companion_instance = null;
  function Text$Companion_getInstance() {
    if (Text$Companion_instance === null) {
      new Text$Companion();
    }
    return Text$Companion_instance;
  }
  Text.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Text',
    interfaces: [Clause]
  };
  Text.prototype.component1 = function () {
    return this.text;
  };
  Text.prototype.copy_61zpoe$ = function (text) {
    return new Text(text === void 0 ? this.text : text);
  };
  Text.prototype.toString = function () {
    return 'Text(text=' + Kotlin.toString(this.text) + ')';
  };
  Text.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.text) | 0;
    return result;
  };
  Text.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.text, other.text))));
  };
  function ExistsGroup(existsSection, suchThatSection) {
    ExistsGroup$Companion_getInstance();
    Clause.call(this);
    this.existsSection = existsSection;
    this.suchThatSection = suchThatSection;
  }
  ExistsGroup.prototype.forEach_ye21ev$ = function (fn) {
    fn(this.existsSection);
    fn(this.suchThatSection);
  };
  ExistsGroup.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode_0(isArg, indent, [this.existsSection, this.suchThatSection]);
  };
  function ExistsGroup$Companion() {
    ExistsGroup$Companion_instance = this;
  }
  ExistsGroup$Companion.prototype.isExistsGroup_rk66c5$ = function (node) {
    return firstSectionMatchesName(node, 'exists');
  };
  ExistsGroup$Companion.prototype.validate_rk66c5$ = function (node) {
    return validateDoubleSectionGroup(node, 'exists', getCallableRef('validate', function ($receiver, node) {
      return $receiver.validate_rk66c5$(node);
    }.bind(null, ExistsSection$Companion_getInstance())), 'suchThat', getCallableRef('validate', function ($receiver, node) {
      return $receiver.validate_rk66c5$(node);
    }.bind(null, SuchThatSection$Companion_getInstance())), getCallableRef('ExistsGroup', function (existsSection, suchThatSection) {
      return new ExistsGroup(existsSection, suchThatSection);
    }));
  };
  ExistsGroup$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var ExistsGroup$Companion_instance = null;
  function ExistsGroup$Companion_getInstance() {
    if (ExistsGroup$Companion_instance === null) {
      new ExistsGroup$Companion();
    }
    return ExistsGroup$Companion_instance;
  }
  ExistsGroup.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ExistsGroup',
    interfaces: [Clause]
  };
  ExistsGroup.prototype.component1 = function () {
    return this.existsSection;
  };
  ExistsGroup.prototype.component2 = function () {
    return this.suchThatSection;
  };
  ExistsGroup.prototype.copy_7tq58m$ = function (existsSection, suchThatSection) {
    return new ExistsGroup(existsSection === void 0 ? this.existsSection : existsSection, suchThatSection === void 0 ? this.suchThatSection : suchThatSection);
  };
  ExistsGroup.prototype.toString = function () {
    return 'ExistsGroup(existsSection=' + Kotlin.toString(this.existsSection) + (', suchThatSection=' + Kotlin.toString(this.suchThatSection)) + ')';
  };
  ExistsGroup.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.existsSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.suchThatSection) | 0;
    return result;
  };
  ExistsGroup.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.existsSection, other.existsSection) && Kotlin.equals(this.suchThatSection, other.suchThatSection)))));
  };
  function IfGroup(ifSection, thenSection) {
    IfGroup$Companion_getInstance();
    Clause.call(this);
    this.ifSection = ifSection;
    this.thenSection = thenSection;
  }
  IfGroup.prototype.forEach_ye21ev$ = function (fn) {
    fn(this.ifSection);
    fn(this.thenSection);
  };
  IfGroup.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode_0(isArg, indent, [this.ifSection, this.thenSection]);
  };
  function IfGroup$Companion() {
    IfGroup$Companion_instance = this;
  }
  IfGroup$Companion.prototype.isIfGroup_rk66c5$ = function (node) {
    return firstSectionMatchesName(node, 'if');
  };
  IfGroup$Companion.prototype.validate_rk66c5$ = function (node) {
    return validateDoubleSectionGroup(node, 'if', getCallableRef('validate', function ($receiver, node) {
      return $receiver.validate_rk66c5$(node);
    }.bind(null, IfSection$Companion_getInstance())), 'then', getCallableRef('validate', function ($receiver, node) {
      return $receiver.validate_rk66c5$(node);
    }.bind(null, ThenSection$Companion_getInstance())), getCallableRef('IfGroup', function (ifSection, thenSection) {
      return new IfGroup(ifSection, thenSection);
    }));
  };
  IfGroup$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var IfGroup$Companion_instance = null;
  function IfGroup$Companion_getInstance() {
    if (IfGroup$Companion_instance === null) {
      new IfGroup$Companion();
    }
    return IfGroup$Companion_instance;
  }
  IfGroup.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'IfGroup',
    interfaces: [Clause]
  };
  IfGroup.prototype.component1 = function () {
    return this.ifSection;
  };
  IfGroup.prototype.component2 = function () {
    return this.thenSection;
  };
  IfGroup.prototype.copy_7g3l9w$ = function (ifSection, thenSection) {
    return new IfGroup(ifSection === void 0 ? this.ifSection : ifSection, thenSection === void 0 ? this.thenSection : thenSection);
  };
  IfGroup.prototype.toString = function () {
    return 'IfGroup(ifSection=' + Kotlin.toString(this.ifSection) + (', thenSection=' + Kotlin.toString(this.thenSection)) + ')';
  };
  IfGroup.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.ifSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.thenSection) | 0;
    return result;
  };
  IfGroup.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.ifSection, other.ifSection) && Kotlin.equals(this.thenSection, other.thenSection)))));
  };
  function IffGroup(iffSection, thenSection) {
    IffGroup$Companion_getInstance();
    Clause.call(this);
    this.iffSection = iffSection;
    this.thenSection = thenSection;
  }
  IffGroup.prototype.forEach_ye21ev$ = function (fn) {
    fn(this.iffSection);
    fn(this.thenSection);
  };
  IffGroup.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode_0(isArg, indent, [this.iffSection, this.thenSection]);
  };
  function IffGroup$Companion() {
    IffGroup$Companion_instance = this;
  }
  IffGroup$Companion.prototype.isIffGroup_rk66c5$ = function (node) {
    return firstSectionMatchesName(node, 'iff');
  };
  IffGroup$Companion.prototype.validate_rk66c5$ = function (node) {
    return validateDoubleSectionGroup(node, 'iff', getCallableRef('validate', function ($receiver, node) {
      return $receiver.validate_rk66c5$(node);
    }.bind(null, IffSection$Companion_getInstance())), 'then', getCallableRef('validate', function ($receiver, node) {
      return $receiver.validate_rk66c5$(node);
    }.bind(null, ThenSection$Companion_getInstance())), getCallableRef('IffGroup', function (iffSection, thenSection) {
      return new IffGroup(iffSection, thenSection);
    }));
  };
  IffGroup$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var IffGroup$Companion_instance = null;
  function IffGroup$Companion_getInstance() {
    if (IffGroup$Companion_instance === null) {
      new IffGroup$Companion();
    }
    return IffGroup$Companion_instance;
  }
  IffGroup.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'IffGroup',
    interfaces: [Clause]
  };
  IffGroup.prototype.component1 = function () {
    return this.iffSection;
  };
  IffGroup.prototype.component2 = function () {
    return this.thenSection;
  };
  IffGroup.prototype.copy_g62wye$ = function (iffSection, thenSection) {
    return new IffGroup(iffSection === void 0 ? this.iffSection : iffSection, thenSection === void 0 ? this.thenSection : thenSection);
  };
  IffGroup.prototype.toString = function () {
    return 'IffGroup(iffSection=' + Kotlin.toString(this.iffSection) + (', thenSection=' + Kotlin.toString(this.thenSection)) + ')';
  };
  IffGroup.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.iffSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.thenSection) | 0;
    return result;
  };
  IffGroup.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.iffSection, other.iffSection) && Kotlin.equals(this.thenSection, other.thenSection)))));
  };
  function ForGroup(forSection, whereSection, thenSection) {
    ForGroup$Companion_getInstance();
    Clause.call(this);
    this.forSection = forSection;
    this.whereSection = whereSection;
    this.thenSection = thenSection;
  }
  ForGroup.prototype.forEach_ye21ev$ = function (fn) {
    fn(this.forSection);
    if (this.whereSection != null) {
      fn(this.whereSection);
    }
    fn(this.thenSection);
  };
  ForGroup.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode_0(isArg, indent, [this.forSection, this.whereSection, this.thenSection]);
  };
  function ForGroup$Companion() {
    ForGroup$Companion_instance = this;
  }
  ForGroup$Companion.prototype.isForGroup_rk66c5$ = function (node) {
    return firstSectionMatchesName(node, 'for');
  };
  ForGroup$Companion.prototype.validate_rk66c5$ = function (rawNode) {
    var tmp$;
    var node = rawNode.resolve();
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Group)) {
      errors.add_11rb$(new ParseError('Expected a Group', AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
      return Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
    var sections = node.component1();
    var sectionMap;
    try {
      sectionMap = SectionIdentifier_getInstance().identifySections_b3nzct$(sections, ['for', 'where?', 'then']);
    }
     catch (e) {
      if (Kotlin.isType(e, ParseError)) {
        errors.add_11rb$(new ParseError(e.message, e.row, e.column));
        return Validation$Companion_getInstance().failure_rg4ulb$(errors);
      }
       else
        throw e;
    }
    var forSection = null;
    var forNode = sectionMap.get_11rb$('for');
    var forEvaluation = ForSection$Companion_getInstance().validate_rk66c5$(ensureNotNull(forNode));
    if (forEvaluation.isSuccessful) {
      forSection = forEvaluation.value;
    }
     else {
      errors.addAll_brywnq$(forEvaluation.errors);
    }
    var whereSection = null;
    if (sectionMap.containsKey_11rb$('where')) {
      var where = sectionMap.get_11rb$('where');
      var whereValidation = WhereSection$Companion_getInstance().validate_rk66c5$(ensureNotNull(where));
      if (whereValidation.isSuccessful) {
        whereSection = ensureNotNull(whereValidation.value);
      }
       else {
        errors.addAll_brywnq$(whereValidation.errors);
      }
    }
    var thenSection = null;
    var then = sectionMap.get_11rb$('then');
    var thenValidation = ThenSection$Companion_getInstance().validate_rk66c5$(ensureNotNull(then));
    if (thenValidation.isSuccessful) {
      thenSection = thenValidation.value;
    }
     else {
      errors.addAll_brywnq$(thenValidation.errors);
    }
    if (!errors.isEmpty()) {
      tmp$ = Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
     else
      tmp$ = Validation$Companion_getInstance().success_mh5how$(new ForGroup(ensureNotNull(forSection), whereSection, ensureNotNull(thenSection)));
    return tmp$;
  };
  ForGroup$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var ForGroup$Companion_instance = null;
  function ForGroup$Companion_getInstance() {
    if (ForGroup$Companion_instance === null) {
      new ForGroup$Companion();
    }
    return ForGroup$Companion_instance;
  }
  ForGroup.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ForGroup',
    interfaces: [Clause]
  };
  ForGroup.prototype.component1 = function () {
    return this.forSection;
  };
  ForGroup.prototype.component2 = function () {
    return this.whereSection;
  };
  ForGroup.prototype.component3 = function () {
    return this.thenSection;
  };
  ForGroup.prototype.copy_rxrvck$ = function (forSection, whereSection, thenSection) {
    return new ForGroup(forSection === void 0 ? this.forSection : forSection, whereSection === void 0 ? this.whereSection : whereSection, thenSection === void 0 ? this.thenSection : thenSection);
  };
  ForGroup.prototype.toString = function () {
    return 'ForGroup(forSection=' + Kotlin.toString(this.forSection) + (', whereSection=' + Kotlin.toString(this.whereSection)) + (', thenSection=' + Kotlin.toString(this.thenSection)) + ')';
  };
  ForGroup.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.forSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.whereSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.thenSection) | 0;
    return result;
  };
  ForGroup.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.forSection, other.forSection) && Kotlin.equals(this.whereSection, other.whereSection) && Kotlin.equals(this.thenSection, other.thenSection)))));
  };
  function NotGroup(notSection) {
    NotGroup$Companion_getInstance();
    Clause.call(this);
    this.notSection = notSection;
  }
  NotGroup.prototype.forEach_ye21ev$ = function (fn) {
    fn(this.notSection);
  };
  NotGroup.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return this.notSection.toCode_eltk6l$(isArg, indent);
  };
  function NotGroup$Companion() {
    NotGroup$Companion_instance = this;
  }
  NotGroup$Companion.prototype.isNotGroup_rk66c5$ = function (node) {
    return firstSectionMatchesName(node, 'not');
  };
  NotGroup$Companion.prototype.validate_rk66c5$ = function (node) {
    return validateSingleSectionGroup(node, 'not', getCallableRef('NotGroup', function (notSection) {
      return new NotGroup(notSection);
    }), getCallableRef('validate', function ($receiver, node) {
      return $receiver.validate_rk66c5$(node);
    }.bind(null, NotSection$Companion_getInstance())));
  };
  NotGroup$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var NotGroup$Companion_instance = null;
  function NotGroup$Companion_getInstance() {
    if (NotGroup$Companion_instance === null) {
      new NotGroup$Companion();
    }
    return NotGroup$Companion_instance;
  }
  NotGroup.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'NotGroup',
    interfaces: [Clause]
  };
  NotGroup.prototype.component1 = function () {
    return this.notSection;
  };
  NotGroup.prototype.copy_u5ve3h$ = function (notSection) {
    return new NotGroup(notSection === void 0 ? this.notSection : notSection);
  };
  NotGroup.prototype.toString = function () {
    return 'NotGroup(notSection=' + Kotlin.toString(this.notSection) + ')';
  };
  NotGroup.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.notSection) | 0;
    return result;
  };
  NotGroup.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.notSection, other.notSection))));
  };
  function OrGroup(orSection) {
    OrGroup$Companion_getInstance();
    Clause.call(this);
    this.orSection = orSection;
  }
  OrGroup.prototype.forEach_ye21ev$ = function (fn) {
    fn(this.orSection);
  };
  OrGroup.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return this.orSection.toCode_eltk6l$(isArg, indent);
  };
  function OrGroup$Companion() {
    OrGroup$Companion_instance = this;
  }
  OrGroup$Companion.prototype.isOrGroup_rk66c5$ = function (node) {
    return firstSectionMatchesName(node, 'or');
  };
  OrGroup$Companion.prototype.validate_rk66c5$ = function (node) {
    return validateSingleSectionGroup(node, 'or', getCallableRef('OrGroup', function (orSection) {
      return new OrGroup(orSection);
    }), getCallableRef('validate', function ($receiver, node) {
      return $receiver.validate_rk66c5$(node);
    }.bind(null, OrSection$Companion_getInstance())));
  };
  OrGroup$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var OrGroup$Companion_instance = null;
  function OrGroup$Companion_getInstance() {
    if (OrGroup$Companion_instance === null) {
      new OrGroup$Companion();
    }
    return OrGroup$Companion_instance;
  }
  OrGroup.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'OrGroup',
    interfaces: [Clause]
  };
  OrGroup.prototype.component1 = function () {
    return this.orSection;
  };
  OrGroup.prototype.copy_3mu1ap$ = function (orSection) {
    return new OrGroup(orSection === void 0 ? this.orSection : orSection);
  };
  OrGroup.prototype.toString = function () {
    return 'OrGroup(orSection=' + Kotlin.toString(this.orSection) + ')';
  };
  OrGroup.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.orSection) | 0;
    return result;
  };
  OrGroup.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.orSection, other.orSection))));
  };
  function firstSectionMatchesName(node, name) {
    var tmp$;
    if (!Kotlin.isType(node, Group)) {
      return false;
    }
    var sections = node.component1();
    if (sections.isEmpty()) {
      tmp$ = false;
    }
     else
      tmp$ = equals(sections.get_za3lpa$(0).name.text, name);
    return tmp$;
  }
  function validateSingleSectionGroup(rawNode, sectionName, buildGroup, validateSection) {
    var tmp$;
    var node = rawNode.resolve();
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Group)) {
      errors.add_11rb$(new ParseError('Expected a Group', AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
      return Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
    var sections = node.component1();
    var sectionMap;
    try {
      sectionMap = SectionIdentifier_getInstance().identifySections_b3nzct$(sections, [sectionName]);
    }
     catch (e) {
      if (Kotlin.isType(e, ParseError)) {
        errors.add_11rb$(new ParseError(e.message, e.row, e.column));
        return Validation$Companion_getInstance().failure_rg4ulb$(errors);
      }
       else
        throw e;
    }
    var section = null;
    var sect = sectionMap.get_11rb$(sectionName);
    var validation = validateSection(ensureNotNull(sect));
    if (validation.isSuccessful) {
      section = validation.value;
    }
     else {
      errors.addAll_brywnq$(validation.errors);
    }
    if (!errors.isEmpty()) {
      tmp$ = Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
     else
      tmp$ = Validation$Companion_getInstance().success_mh5how$(buildGroup(ensureNotNull(section)));
    return tmp$;
  }
  function validateDoubleSectionGroup(rawNode, section1Name, validateSection1, section2Name, validateSection2, buildGroup) {
    var tmp$;
    var node = rawNode.resolve();
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Group)) {
      errors.add_11rb$(new ParseError('Expected a Group', AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
      return Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
    var sections = node.component1();
    var sectionMap;
    try {
      sectionMap = SectionIdentifier_getInstance().identifySections_b3nzct$(sections, [section1Name, section2Name]);
    }
     catch (e) {
      if (Kotlin.isType(e, ParseError)) {
        errors.add_11rb$(new ParseError(e.message, e.row, e.column));
        return Validation$Companion_getInstance().failure_rg4ulb$(errors);
      }
       else
        throw e;
    }
    var section1 = null;
    var sect1 = sectionMap.get_11rb$(section1Name);
    var section1Validation = validateSection1(ensureNotNull(sect1));
    if (section1Validation.isSuccessful) {
      section1 = section1Validation.value;
    }
     else {
      errors.addAll_brywnq$(section1Validation.errors);
    }
    var section2 = null;
    var sect2 = sectionMap.get_11rb$(section2Name);
    var section2Validation = validateSection2(ensureNotNull(sect2));
    if (section2Validation.isSuccessful) {
      section2 = section2Validation.value;
    }
     else {
      errors.addAll_brywnq$(section2Validation.errors);
    }
    if (!errors.isEmpty()) {
      tmp$ = Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
     else
      tmp$ = Validation$Companion_getInstance().success_mh5how$(buildGroup(ensureNotNull(section1), ensureNotNull(section2)));
    return tmp$;
  }
  function validateWrappedNode(rawNode, expectedType, checkType, build) {
    var node = rawNode.resolve();
    var base = checkType(node);
    if (base == null) {
      return Validation$Companion_getInstance().failure_rg4ulb$(listOf(new ParseError('Cannot convert to a ' + expectedType, AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node))));
    }
    return Validation$Companion_getInstance().success_mh5how$(build(base));
  }
  function toCode(isArg, indent, chalkTalkNode) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, ''));
    builder.append_gw00v9$(chalkTalkNode.toCode());
    return builder.toString();
  }
  function toCode_0(isArg, indent, sections) {
    var builder = StringBuilder_init();
    for (var i = 0; i < sections.length; i++) {
      var sect = sections[i];
      if (sect != null) {
        builder.append_gw00v9$(sect.toCode_eltk6l$(isArg, indent));
        if (i !== (sections.length - 1 | 0)) {
          builder.append_s8itvh$(10);
        }
      }
    }
    return builder.toString();
  }
  function ClauseListSection(name, clauses) {
    this.name = name;
    this.clauses = clauses;
  }
  ClauseListSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ClauseListSection',
    interfaces: []
  };
  ClauseListSection.prototype.component1 = function () {
    return this.name;
  };
  ClauseListSection.prototype.component2 = function () {
    return this.clauses;
  };
  ClauseListSection.prototype.copy_81npfn$ = function (name, clauses) {
    return new ClauseListSection(name === void 0 ? this.name : name, clauses === void 0 ? this.clauses : clauses);
  };
  ClauseListSection.prototype.toString = function () {
    return 'ClauseListSection(name=' + Kotlin.toString(this.name) + (', clauses=' + Kotlin.toString(this.clauses)) + ')';
  };
  ClauseListSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.name) | 0;
    result = result * 31 + Kotlin.hashCode(this.clauses) | 0;
    return result;
  };
  ClauseListSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.name, other.name) && Kotlin.equals(this.clauses, other.clauses)))));
  };
  function ClauseListValidator() {
    ClauseListValidator_instance = this;
  }
  ClauseListValidator.prototype.validate_ro44ti$ = function (rawNode, expectedName, builder) {
    var node = rawNode.resolve();
    var validation = this.validate_0(node, expectedName);
    if (!validation.isSuccessful) {
      return Validation$Companion_getInstance().failure_rg4ulb$(validation.errors);
    }
    var clauses = ensureNotNull(validation.value).clauses;
    return Validation$Companion_getInstance().success_mh5how$(builder(clauses));
  };
  ClauseListValidator.prototype.validate_0 = function (node, expectedName) {
    var tmp$, tmp$_0, tmp$_1;
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Section)) {
      errors.add_11rb$(new ParseError('Expected a Section', AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
    }
    var tmp$_2 = Kotlin.isType(tmp$ = node, Section) ? tmp$ : throwCCE();
    var name = tmp$_2.component1()
    , args = tmp$_2.component2();
    if (!equals(name.text, expectedName)) {
      errors.add_11rb$(new ParseError('Expected a Section with name ' + expectedName + ' but found ' + name.text, AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
    }
    if (args.isEmpty()) {
      errors.add_11rb$(new ParseError("Section '" + name.text + "' requires at least one argument.", AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
    }
    var clauses = ArrayList_init();
    tmp$_0 = args.iterator();
    while (tmp$_0.hasNext()) {
      var arg = tmp$_0.next();
      var validation = Clause$Companion_getInstance().validate_rk66c5$(arg);
      if (validation.isSuccessful) {
        clauses.add_11rb$(ensureNotNull(validation.value));
      }
       else {
        errors.addAll_brywnq$(validation.errors);
      }
    }
    if (!errors.isEmpty()) {
      tmp$_1 = Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
     else
      tmp$_1 = Validation$Companion_getInstance().success_mh5how$(new ClauseListSection(name.text, clauses));
    return tmp$_1;
  };
  ClauseListValidator.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'ClauseListValidator',
    interfaces: []
  };
  var ClauseListValidator_instance = null;
  function ClauseListValidator_getInstance() {
    if (ClauseListValidator_instance === null) {
      new ClauseListValidator();
    }
    return ClauseListValidator_instance;
  }
  function Phase2Node() {
  }
  Phase2Node.$metadata$ = {
    kind: Kind_INTERFACE,
    simpleName: 'Phase2Node',
    interfaces: []
  };
  function Document(defines, represents, results, axioms, conjectures) {
    Document$Companion_getInstance();
    this.defines = defines;
    this.represents = represents;
    this.results = results;
    this.axioms = axioms;
    this.conjectures = conjectures;
  }
  Document.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.defines.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
    var tmp$_0;
    tmp$_0 = this.represents.iterator();
    while (tmp$_0.hasNext()) {
      var element_0 = tmp$_0.next();
      fn(element_0);
    }
    var tmp$_1;
    tmp$_1 = this.results.iterator();
    while (tmp$_1.hasNext()) {
      var element_1 = tmp$_1.next();
      fn(element_1);
    }
    var tmp$_2;
    tmp$_2 = this.axioms.iterator();
    while (tmp$_2.hasNext()) {
      var element_2 = tmp$_2.next();
      fn(element_2);
    }
    var tmp$_3;
    tmp$_3 = this.conjectures.iterator();
    while (tmp$_3.hasNext()) {
      var element_3 = tmp$_3.next();
      fn(element_3);
    }
  };
  Document.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var tmp$, tmp$_0, tmp$_1, tmp$_2, tmp$_3;
    var builder = StringBuilder_init();
    tmp$ = this.defines.iterator();
    while (tmp$.hasNext()) {
      var grp = tmp$.next();
      builder.append_gw00v9$(grp.toCode_eltk6l$(false, 0));
      builder.append_gw00v9$('\n\n\n');
    }
    tmp$_0 = this.represents.iterator();
    while (tmp$_0.hasNext()) {
      var grp_0 = tmp$_0.next();
      builder.append_gw00v9$(grp_0.toCode_eltk6l$(false, 0));
      builder.append_gw00v9$('\n\n\n');
    }
    tmp$_1 = this.axioms.iterator();
    while (tmp$_1.hasNext()) {
      var grp_1 = tmp$_1.next();
      builder.append_gw00v9$(grp_1.toCode_eltk6l$(false, 0));
      builder.append_gw00v9$('\n\n\n');
    }
    tmp$_2 = this.conjectures.iterator();
    while (tmp$_2.hasNext()) {
      var grp_2 = tmp$_2.next();
      builder.append_gw00v9$(grp_2.toCode_eltk6l$(false, 0));
      builder.append_gw00v9$('\n\n\n');
    }
    tmp$_3 = this.results.iterator();
    while (tmp$_3.hasNext()) {
      var grp_3 = tmp$_3.next();
      builder.append_gw00v9$(grp_3.toCode_eltk6l$(false, 0));
      builder.append_gw00v9$('\n\n\n');
    }
    return builder.toString();
  };
  function Document$Companion() {
    Document$Companion_instance = this;
  }
  Document$Companion.prototype.validate_rk66c5$ = function (rawNode) {
    var tmp$, tmp$_0;
    var node = rawNode.resolve();
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Root)) {
      errors.add_11rb$(new ParseError('Expected a Root', AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
      return Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
    var defines = ArrayList_init();
    var represents = ArrayList_init();
    var results = ArrayList_init();
    var axioms = ArrayList_init();
    var conjectures = ArrayList_init();
    var groups = node.component1();
    tmp$ = groups.iterator();
    while (tmp$.hasNext()) {
      var group = tmp$.next();
      if (ResultGroup$Companion_getInstance().isResultGroup_rk66c5$(group)) {
        var resultValidation = ResultGroup$Companion_getInstance().validate_hzi7mn$(group);
        if (resultValidation.isSuccessful) {
          results.add_11rb$(ensureNotNull(resultValidation.value));
        }
         else {
          errors.addAll_brywnq$(resultValidation.errors);
        }
      }
       else if (AxiomGroup$Companion_getInstance().isAxiomGroup_rk66c5$(group)) {
        var axiomValidation = AxiomGroup$Companion_getInstance().validate_hzi7mn$(group);
        if (axiomValidation.isSuccessful) {
          axioms.add_11rb$(ensureNotNull(axiomValidation.value));
        }
         else {
          errors.addAll_brywnq$(axiomValidation.errors);
        }
      }
       else if (ConjectureGroup$Companion_getInstance().isConjectureGroup_rk66c5$(group)) {
        var conjectureValidation = ConjectureGroup$Companion_getInstance().validate_hzi7mn$(group);
        if (conjectureValidation.isSuccessful) {
          conjectures.add_11rb$(ensureNotNull(conjectureValidation.value));
        }
         else {
          errors.addAll_brywnq$(conjectureValidation.errors);
        }
      }
       else if (DefinesGroup$Companion_getInstance().isDefinesGroup_rk66c5$(group)) {
        var definesValidation = DefinesGroup$Companion_getInstance().validate_hzi7mn$(group);
        if (definesValidation.isSuccessful) {
          defines.add_11rb$(ensureNotNull(definesValidation.value));
        }
         else {
          errors.addAll_brywnq$(definesValidation.errors);
        }
      }
       else if (RepresentsGroup$Companion_getInstance().isRepresentsGroup_rk66c5$(group)) {
        var representsValidation = RepresentsGroup$Companion_getInstance().validate_hzi7mn$(group);
        if (representsValidation.isSuccessful) {
          represents.add_11rb$(ensureNotNull(representsValidation.value));
        }
         else {
          errors.addAll_brywnq$(representsValidation.errors);
        }
      }
       else {
        errors.add_11rb$(new ParseError('Expected a Result or Defines but found ' + group.toCode(), AstUtils_getInstance().getRow_rk66c5$(group), AstUtils_getInstance().getColumn_rk66c5$(group)));
      }
    }
    if (!errors.isEmpty()) {
      tmp$_0 = Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
     else
      tmp$_0 = Validation$Companion_getInstance().success_mh5how$(new Document(defines, represents, results, axioms, conjectures));
    return tmp$_0;
  };
  Document$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var Document$Companion_instance = null;
  function Document$Companion_getInstance() {
    if (Document$Companion_instance === null) {
      new Document$Companion();
    }
    return Document$Companion_instance;
  }
  Document.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Document',
    interfaces: [Phase2Node]
  };
  Document.prototype.component1 = function () {
    return this.defines;
  };
  Document.prototype.component2 = function () {
    return this.represents;
  };
  Document.prototype.component3 = function () {
    return this.results;
  };
  Document.prototype.component4 = function () {
    return this.axioms;
  };
  Document.prototype.component5 = function () {
    return this.conjectures;
  };
  Document.prototype.copy_e3tw37$ = function (defines, represents, results, axioms, conjectures) {
    return new Document(defines === void 0 ? this.defines : defines, represents === void 0 ? this.represents : represents, results === void 0 ? this.results : results, axioms === void 0 ? this.axioms : axioms, conjectures === void 0 ? this.conjectures : conjectures);
  };
  Document.prototype.toString = function () {
    return 'Document(defines=' + Kotlin.toString(this.defines) + (', represents=' + Kotlin.toString(this.represents)) + (', results=' + Kotlin.toString(this.results)) + (', axioms=' + Kotlin.toString(this.axioms)) + (', conjectures=' + Kotlin.toString(this.conjectures)) + ')';
  };
  Document.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.defines) | 0;
    result = result * 31 + Kotlin.hashCode(this.represents) | 0;
    result = result * 31 + Kotlin.hashCode(this.results) | 0;
    result = result * 31 + Kotlin.hashCode(this.axioms) | 0;
    result = result * 31 + Kotlin.hashCode(this.conjectures) | 0;
    return result;
  };
  Document.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.defines, other.defines) && Kotlin.equals(this.represents, other.represents) && Kotlin.equals(this.results, other.results) && Kotlin.equals(this.axioms, other.axioms) && Kotlin.equals(this.conjectures, other.conjectures)))));
  };
  function DefinesGroup(signature, id, definesSection, assumingSection, meansSection, aliasSection, metaDataSection) {
    DefinesGroup$Companion_getInstance();
    this.signature = signature;
    this.id = id;
    this.definesSection = definesSection;
    this.assumingSection = assumingSection;
    this.meansSection = meansSection;
    this.aliasSection = aliasSection;
    this.metaDataSection = metaDataSection;
  }
  DefinesGroup.prototype.forEach_ye21ev$ = function (fn) {
    fn(this.id);
    fn(this.definesSection);
    if (this.assumingSection != null) {
      fn(this.assumingSection);
    }
    fn(this.meansSection);
    if (this.metaDataSection != null) {
      fn(this.metaDataSection);
    }
  };
  DefinesGroup.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode_1(isArg, indent, this.id, [this.definesSection, this.assumingSection, this.meansSection, this.metaDataSection]);
  };
  function DefinesGroup$Companion() {
    DefinesGroup$Companion_instance = this;
  }
  DefinesGroup$Companion.prototype.isDefinesGroup_rk66c5$ = function (node) {
    return firstSectionMatchesName(node, 'Defines');
  };
  DefinesGroup$Companion.prototype.validate_hzi7mn$ = function (groupNode) {
    return validateDefinesLikeGroup(groupNode, 'Defines', getCallableRef('validate', function ($receiver, node) {
      return $receiver.validate_rk66c5$(node);
    }.bind(null, DefinesSection$Companion_getInstance())), 'means', getCallableRef('validate', function ($receiver, node) {
      return $receiver.validate_rk66c5$(node);
    }.bind(null, MeansSection$Companion_getInstance())), getCallableRef('DefinesGroup', function (signature, id, definesSection, assumingSection, meansSection, aliasSection, metaDataSection) {
      return new DefinesGroup(signature, id, definesSection, assumingSection, meansSection, aliasSection, metaDataSection);
    }));
  };
  DefinesGroup$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var DefinesGroup$Companion_instance = null;
  function DefinesGroup$Companion_getInstance() {
    if (DefinesGroup$Companion_instance === null) {
      new DefinesGroup$Companion();
    }
    return DefinesGroup$Companion_instance;
  }
  DefinesGroup.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'DefinesGroup',
    interfaces: [Phase2Node]
  };
  DefinesGroup.prototype.component1 = function () {
    return this.signature;
  };
  DefinesGroup.prototype.component2 = function () {
    return this.id;
  };
  DefinesGroup.prototype.component3 = function () {
    return this.definesSection;
  };
  DefinesGroup.prototype.component4 = function () {
    return this.assumingSection;
  };
  DefinesGroup.prototype.component5 = function () {
    return this.meansSection;
  };
  DefinesGroup.prototype.component6 = function () {
    return this.aliasSection;
  };
  DefinesGroup.prototype.component7 = function () {
    return this.metaDataSection;
  };
  DefinesGroup.prototype.copy_j81lf8$ = function (signature, id, definesSection, assumingSection, meansSection, aliasSection, metaDataSection) {
    return new DefinesGroup(signature === void 0 ? this.signature : signature, id === void 0 ? this.id : id, definesSection === void 0 ? this.definesSection : definesSection, assumingSection === void 0 ? this.assumingSection : assumingSection, meansSection === void 0 ? this.meansSection : meansSection, aliasSection === void 0 ? this.aliasSection : aliasSection, metaDataSection === void 0 ? this.metaDataSection : metaDataSection);
  };
  DefinesGroup.prototype.toString = function () {
    return 'DefinesGroup(signature=' + Kotlin.toString(this.signature) + (', id=' + Kotlin.toString(this.id)) + (', definesSection=' + Kotlin.toString(this.definesSection)) + (', assumingSection=' + Kotlin.toString(this.assumingSection)) + (', meansSection=' + Kotlin.toString(this.meansSection)) + (', aliasSection=' + Kotlin.toString(this.aliasSection)) + (', metaDataSection=' + Kotlin.toString(this.metaDataSection)) + ')';
  };
  DefinesGroup.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.signature) | 0;
    result = result * 31 + Kotlin.hashCode(this.id) | 0;
    result = result * 31 + Kotlin.hashCode(this.definesSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.assumingSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.meansSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.aliasSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.metaDataSection) | 0;
    return result;
  };
  DefinesGroup.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.signature, other.signature) && Kotlin.equals(this.id, other.id) && Kotlin.equals(this.definesSection, other.definesSection) && Kotlin.equals(this.assumingSection, other.assumingSection) && Kotlin.equals(this.meansSection, other.meansSection) && Kotlin.equals(this.aliasSection, other.aliasSection) && Kotlin.equals(this.metaDataSection, other.metaDataSection)))));
  };
  function RepresentsGroup(signature, id, representsSection, assumingSection, thatSection, aliasSection, metaDataSection) {
    RepresentsGroup$Companion_getInstance();
    this.signature = signature;
    this.id = id;
    this.representsSection = representsSection;
    this.assumingSection = assumingSection;
    this.thatSection = thatSection;
    this.aliasSection = aliasSection;
    this.metaDataSection = metaDataSection;
  }
  RepresentsGroup.prototype.forEach_ye21ev$ = function (fn) {
    fn(this.id);
    fn(this.representsSection);
    if (this.assumingSection != null) {
      fn(this.assumingSection);
    }
    fn(this.thatSection);
    if (this.metaDataSection != null) {
      fn(this.metaDataSection);
    }
  };
  RepresentsGroup.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode_1(isArg, indent, this.id, [this.representsSection, this.assumingSection, this.thatSection, this.metaDataSection]);
  };
  function RepresentsGroup$Companion() {
    RepresentsGroup$Companion_instance = this;
  }
  RepresentsGroup$Companion.prototype.isRepresentsGroup_rk66c5$ = function (node) {
    return firstSectionMatchesName(node, 'Represents');
  };
  RepresentsGroup$Companion.prototype.validate_hzi7mn$ = function (groupNode) {
    return validateDefinesLikeGroup(groupNode, 'Represents', getCallableRef('validate', function ($receiver, node) {
      return $receiver.validate_rk66c5$(node);
    }.bind(null, RepresentsSection$Companion_getInstance())), 'that', getCallableRef('validate', function ($receiver, node) {
      return $receiver.validate_rk66c5$(node);
    }.bind(null, ThatSection$Companion_getInstance())), getCallableRef('RepresentsGroup', function (signature, id, representsSection, assumingSection, thatSection, aliasSection, metaDataSection) {
      return new RepresentsGroup(signature, id, representsSection, assumingSection, thatSection, aliasSection, metaDataSection);
    }));
  };
  RepresentsGroup$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var RepresentsGroup$Companion_instance = null;
  function RepresentsGroup$Companion_getInstance() {
    if (RepresentsGroup$Companion_instance === null) {
      new RepresentsGroup$Companion();
    }
    return RepresentsGroup$Companion_instance;
  }
  RepresentsGroup.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'RepresentsGroup',
    interfaces: [Phase2Node]
  };
  RepresentsGroup.prototype.component1 = function () {
    return this.signature;
  };
  RepresentsGroup.prototype.component2 = function () {
    return this.id;
  };
  RepresentsGroup.prototype.component3 = function () {
    return this.representsSection;
  };
  RepresentsGroup.prototype.component4 = function () {
    return this.assumingSection;
  };
  RepresentsGroup.prototype.component5 = function () {
    return this.thatSection;
  };
  RepresentsGroup.prototype.component6 = function () {
    return this.aliasSection;
  };
  RepresentsGroup.prototype.component7 = function () {
    return this.metaDataSection;
  };
  RepresentsGroup.prototype.copy_cyc0tq$ = function (signature, id, representsSection, assumingSection, thatSection, aliasSection, metaDataSection) {
    return new RepresentsGroup(signature === void 0 ? this.signature : signature, id === void 0 ? this.id : id, representsSection === void 0 ? this.representsSection : representsSection, assumingSection === void 0 ? this.assumingSection : assumingSection, thatSection === void 0 ? this.thatSection : thatSection, aliasSection === void 0 ? this.aliasSection : aliasSection, metaDataSection === void 0 ? this.metaDataSection : metaDataSection);
  };
  RepresentsGroup.prototype.toString = function () {
    return 'RepresentsGroup(signature=' + Kotlin.toString(this.signature) + (', id=' + Kotlin.toString(this.id)) + (', representsSection=' + Kotlin.toString(this.representsSection)) + (', assumingSection=' + Kotlin.toString(this.assumingSection)) + (', thatSection=' + Kotlin.toString(this.thatSection)) + (', aliasSection=' + Kotlin.toString(this.aliasSection)) + (', metaDataSection=' + Kotlin.toString(this.metaDataSection)) + ')';
  };
  RepresentsGroup.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.signature) | 0;
    result = result * 31 + Kotlin.hashCode(this.id) | 0;
    result = result * 31 + Kotlin.hashCode(this.representsSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.assumingSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.thatSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.aliasSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.metaDataSection) | 0;
    return result;
  };
  RepresentsGroup.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.signature, other.signature) && Kotlin.equals(this.id, other.id) && Kotlin.equals(this.representsSection, other.representsSection) && Kotlin.equals(this.assumingSection, other.assumingSection) && Kotlin.equals(this.thatSection, other.thatSection) && Kotlin.equals(this.aliasSection, other.aliasSection) && Kotlin.equals(this.metaDataSection, other.metaDataSection)))));
  };
  function ResultGroup(resultSection, aliasSection, metaDataSection) {
    ResultGroup$Companion_getInstance();
    this.resultSection = resultSection;
    this.aliasSection = aliasSection;
    this.metaDataSection = metaDataSection;
  }
  ResultGroup.prototype.forEach_ye21ev$ = function (fn) {
    fn(this.resultSection);
    if (this.metaDataSection != null) {
      fn(this.metaDataSection);
    }
  };
  ResultGroup.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode_1(isArg, indent, null, [this.resultSection, this.metaDataSection]);
  };
  function ResultGroup$Companion() {
    ResultGroup$Companion_instance = this;
  }
  ResultGroup$Companion.prototype.isResultGroup_rk66c5$ = function (node) {
    return firstSectionMatchesName(node, 'Result');
  };
  ResultGroup$Companion.prototype.validate_hzi7mn$ = function (groupNode) {
    return validateResultLikeGroup(groupNode, 'Result', getCallableRef('validate', function ($receiver, node) {
      return $receiver.validate_rk66c5$(node);
    }.bind(null, ResultSection$Companion_getInstance())), getCallableRef('ResultGroup', function (resultSection, aliasSection, metaDataSection) {
      return new ResultGroup(resultSection, aliasSection, metaDataSection);
    }));
  };
  ResultGroup$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var ResultGroup$Companion_instance = null;
  function ResultGroup$Companion_getInstance() {
    if (ResultGroup$Companion_instance === null) {
      new ResultGroup$Companion();
    }
    return ResultGroup$Companion_instance;
  }
  ResultGroup.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ResultGroup',
    interfaces: [Phase2Node]
  };
  ResultGroup.prototype.component1 = function () {
    return this.resultSection;
  };
  ResultGroup.prototype.component2 = function () {
    return this.aliasSection;
  };
  ResultGroup.prototype.component3 = function () {
    return this.metaDataSection;
  };
  ResultGroup.prototype.copy_bjcldw$ = function (resultSection, aliasSection, metaDataSection) {
    return new ResultGroup(resultSection === void 0 ? this.resultSection : resultSection, aliasSection === void 0 ? this.aliasSection : aliasSection, metaDataSection === void 0 ? this.metaDataSection : metaDataSection);
  };
  ResultGroup.prototype.toString = function () {
    return 'ResultGroup(resultSection=' + Kotlin.toString(this.resultSection) + (', aliasSection=' + Kotlin.toString(this.aliasSection)) + (', metaDataSection=' + Kotlin.toString(this.metaDataSection)) + ')';
  };
  ResultGroup.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.resultSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.aliasSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.metaDataSection) | 0;
    return result;
  };
  ResultGroup.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.resultSection, other.resultSection) && Kotlin.equals(this.aliasSection, other.aliasSection) && Kotlin.equals(this.metaDataSection, other.metaDataSection)))));
  };
  function AxiomGroup(axiomSection, aliasSection, metaDataSection) {
    AxiomGroup$Companion_getInstance();
    this.axiomSection = axiomSection;
    this.aliasSection = aliasSection;
    this.metaDataSection = metaDataSection;
  }
  AxiomGroup.prototype.forEach_ye21ev$ = function (fn) {
    fn(this.axiomSection);
    if (this.metaDataSection != null) {
      fn(this.metaDataSection);
    }
  };
  AxiomGroup.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode_1(isArg, indent, null, [this.axiomSection, this.metaDataSection]);
  };
  function AxiomGroup$Companion() {
    AxiomGroup$Companion_instance = this;
  }
  AxiomGroup$Companion.prototype.isAxiomGroup_rk66c5$ = function (node) {
    return firstSectionMatchesName(node, 'Axiom');
  };
  AxiomGroup$Companion.prototype.validate_hzi7mn$ = function (groupNode) {
    return validateResultLikeGroup(groupNode, 'Axiom', getCallableRef('validate', function ($receiver, node) {
      return $receiver.validate_rk66c5$(node);
    }.bind(null, AxiomSection$Companion_getInstance())), getCallableRef('AxiomGroup', function (axiomSection, aliasSection, metaDataSection) {
      return new AxiomGroup(axiomSection, aliasSection, metaDataSection);
    }));
  };
  AxiomGroup$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var AxiomGroup$Companion_instance = null;
  function AxiomGroup$Companion_getInstance() {
    if (AxiomGroup$Companion_instance === null) {
      new AxiomGroup$Companion();
    }
    return AxiomGroup$Companion_instance;
  }
  AxiomGroup.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'AxiomGroup',
    interfaces: [Phase2Node]
  };
  AxiomGroup.prototype.component1 = function () {
    return this.axiomSection;
  };
  AxiomGroup.prototype.component2 = function () {
    return this.aliasSection;
  };
  AxiomGroup.prototype.component3 = function () {
    return this.metaDataSection;
  };
  AxiomGroup.prototype.copy_bs19l9$ = function (axiomSection, aliasSection, metaDataSection) {
    return new AxiomGroup(axiomSection === void 0 ? this.axiomSection : axiomSection, aliasSection === void 0 ? this.aliasSection : aliasSection, metaDataSection === void 0 ? this.metaDataSection : metaDataSection);
  };
  AxiomGroup.prototype.toString = function () {
    return 'AxiomGroup(axiomSection=' + Kotlin.toString(this.axiomSection) + (', aliasSection=' + Kotlin.toString(this.aliasSection)) + (', metaDataSection=' + Kotlin.toString(this.metaDataSection)) + ')';
  };
  AxiomGroup.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.axiomSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.aliasSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.metaDataSection) | 0;
    return result;
  };
  AxiomGroup.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.axiomSection, other.axiomSection) && Kotlin.equals(this.aliasSection, other.aliasSection) && Kotlin.equals(this.metaDataSection, other.metaDataSection)))));
  };
  function ConjectureGroup(conjectureSection, aliasSection, metaDataSection) {
    ConjectureGroup$Companion_getInstance();
    this.conjectureSection = conjectureSection;
    this.aliasSection = aliasSection;
    this.metaDataSection = metaDataSection;
  }
  ConjectureGroup.prototype.forEach_ye21ev$ = function (fn) {
    fn(this.conjectureSection);
    if (this.metaDataSection != null) {
      fn(this.metaDataSection);
    }
  };
  ConjectureGroup.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode_1(isArg, indent, null, [this.conjectureSection, this.metaDataSection]);
  };
  function ConjectureGroup$Companion() {
    ConjectureGroup$Companion_instance = this;
  }
  ConjectureGroup$Companion.prototype.isConjectureGroup_rk66c5$ = function (node) {
    return firstSectionMatchesName(node, 'Conjecture');
  };
  ConjectureGroup$Companion.prototype.validate_hzi7mn$ = function (groupNode) {
    return validateResultLikeGroup(groupNode, 'Conjecture', getCallableRef('validate', function ($receiver, node) {
      return $receiver.validate_rk66c5$(node);
    }.bind(null, ConjectureSection$Companion_getInstance())), getCallableRef('ConjectureGroup', function (conjectureSection, aliasSection, metaDataSection) {
      return new ConjectureGroup(conjectureSection, aliasSection, metaDataSection);
    }));
  };
  ConjectureGroup$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var ConjectureGroup$Companion_instance = null;
  function ConjectureGroup$Companion_getInstance() {
    if (ConjectureGroup$Companion_instance === null) {
      new ConjectureGroup$Companion();
    }
    return ConjectureGroup$Companion_instance;
  }
  ConjectureGroup.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ConjectureGroup',
    interfaces: [Phase2Node]
  };
  ConjectureGroup.prototype.component1 = function () {
    return this.conjectureSection;
  };
  ConjectureGroup.prototype.component2 = function () {
    return this.aliasSection;
  };
  ConjectureGroup.prototype.component3 = function () {
    return this.metaDataSection;
  };
  ConjectureGroup.prototype.copy_hd94nt$ = function (conjectureSection, aliasSection, metaDataSection) {
    return new ConjectureGroup(conjectureSection === void 0 ? this.conjectureSection : conjectureSection, aliasSection === void 0 ? this.aliasSection : aliasSection, metaDataSection === void 0 ? this.metaDataSection : metaDataSection);
  };
  ConjectureGroup.prototype.toString = function () {
    return 'ConjectureGroup(conjectureSection=' + Kotlin.toString(this.conjectureSection) + (', aliasSection=' + Kotlin.toString(this.aliasSection)) + (', metaDataSection=' + Kotlin.toString(this.metaDataSection)) + ')';
  };
  ConjectureGroup.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.conjectureSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.aliasSection) | 0;
    result = result * 31 + Kotlin.hashCode(this.metaDataSection) | 0;
    return result;
  };
  ConjectureGroup.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.conjectureSection, other.conjectureSection) && Kotlin.equals(this.aliasSection, other.aliasSection) && Kotlin.equals(this.metaDataSection, other.metaDataSection)))));
  };
  function toCode_1(isArg, indent, id, sections) {
    var builder = StringBuilder_init();
    var useAsArg = isArg;
    if (id != null) {
      builder.append_gw00v9$(indentedString(isArg, indent, '[' + id.text + ']' + '\n'));
      useAsArg = false;
    }
    for (var i = 0; i < sections.length; i++) {
      var sect = sections[i];
      if (sect != null) {
        builder.append_gw00v9$(sect.toCode_eltk6l$(useAsArg, indent));
        useAsArg = false;
        if (i !== (sections.length - 1 | 0)) {
          builder.append_s8itvh$(10);
        }
      }
    }
    return builder.toString();
  }
  function validateResultLikeGroup(groupNode, resultLikeName, validateResultLikeSection, buildGroup) {
    var tmp$, tmp$_0;
    var errors = ArrayList_init();
    var group = Kotlin.isType(tmp$ = groupNode.resolve(), Group) ? tmp$ : throwCCE();
    if (group.id != null) {
      errors.add_11rb$(new ParseError('A result, axiom, or conjecture cannot have an Id', AstUtils_getInstance().getRow_rk66c5$(group), AstUtils_getInstance().getColumn_rk66c5$(group)));
    }
    var sections = group.sections;
    var sectionMap;
    try {
      sectionMap = SectionIdentifier_getInstance().identifySections_b3nzct$(sections, [resultLikeName, 'Alias?', 'Metadata?']);
    }
     catch (e) {
      if (Kotlin.isType(e, ParseError)) {
        errors.add_11rb$(new ParseError(e.message, e.row, e.column));
        return Validation$Companion_getInstance().failure_rg4ulb$(errors);
      }
       else
        throw e;
    }
    var resultLike = sectionMap.get_11rb$(resultLikeName);
    var alias = getOrNull(sectionMap, 'Alias');
    var metadata = getOrNull(sectionMap, 'Metadata');
    var resultLikeValidation = validateResultLikeSection(ensureNotNull(resultLike));
    var resultLikeSection = null;
    if (resultLikeValidation.isSuccessful) {
      resultLikeSection = resultLikeValidation.value;
    }
     else {
      errors.addAll_brywnq$(resultLikeValidation.errors);
    }
    var metaDataSection = null;
    if (metadata != null) {
      var metaDataValidation = MetaDataSection$Companion_getInstance().validate_3fjnpj$(metadata);
      if (metaDataValidation.isSuccessful) {
        metaDataSection = ensureNotNull(metaDataValidation.value);
      }
       else {
        errors.addAll_brywnq$(metaDataValidation.errors);
      }
    }
    var aliasSection = null;
    if (alias != null) {
      var aliasValidation = AliasSection$Companion_getInstance().validate_3fjnpj$(alias);
      if (aliasValidation.isSuccessful) {
        aliasSection = ensureNotNull(aliasValidation.value);
      }
       else {
        errors.addAll_brywnq$(aliasValidation.errors);
      }
    }
    if (!errors.isEmpty()) {
      tmp$_0 = Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
     else
      tmp$_0 = Validation$Companion_getInstance().success_mh5how$(buildGroup(ensureNotNull(resultLikeSection), aliasSection, metaDataSection));
    return tmp$_0;
  }
  function validateDefinesLikeGroup(groupNode, definesLikeSectionName, validateDefinesLikeSection, endSectionName, validateEndSection, buildGroup) {
    var tmp$, tmp$_0;
    var errors = ArrayList_init();
    var group = Kotlin.isType(tmp$ = groupNode.resolve(), Group) ? tmp$ : throwCCE();
    var id = null;
    if (group.id != null) {
      var tmp$_1 = group.id;
      var rawText = tmp$_1.component1()
      , row = tmp$_1.component3()
      , column = tmp$_1.component4();
      var endIndex = rawText.length - 1 | 0;
      var statementText = "'" + rawText.substring(1, endIndex) + "'";
      var stmtToken = new ChalkTalkToken(statementText, ChalkTalkTokenType$Statement_getInstance(), row, column);
      var idValidation = Statement$Companion_getInstance().validate_rk66c5$(stmtToken);
      if (idValidation.isSuccessful) {
        id = idValidation.value;
      }
       else {
        errors.addAll_brywnq$(idValidation.errors);
      }
    }
     else {
      errors.add_11rb$(new ParseError('A definition must have an Id', AstUtils_getInstance().getRow_rk66c5$(group), AstUtils_getInstance().getColumn_rk66c5$(group)));
    }
    var sections = group.sections;
    var sectionMap;
    try {
      sectionMap = SectionIdentifier_getInstance().identifySections_b3nzct$(sections, [definesLikeSectionName, 'assuming?', endSectionName, 'Alias?', 'Metadata?']);
    }
     catch (e) {
      if (Kotlin.isType(e, ParseError)) {
        errors.add_11rb$(new ParseError(e.message, e.row, e.column));
        return Validation$Companion_getInstance().failure_rg4ulb$(errors);
      }
       else
        throw e;
    }
    var definesLike = sectionMap.get_11rb$(definesLikeSectionName);
    var assuming = getOrNull(sectionMap, 'assuming');
    var end = sectionMap.get_11rb$(endSectionName);
    var alias = getOrNull(sectionMap, 'Alias');
    var metadata = getOrNull(sectionMap, 'Metadata');
    var definesLikeValidation = validateDefinesLikeSection(ensureNotNull(definesLike));
    var definesLikeSection = null;
    if (definesLikeValidation.isSuccessful) {
      definesLikeSection = definesLikeValidation.value;
    }
     else {
      errors.addAll_brywnq$(definesLikeValidation.errors);
    }
    var assumingSection = null;
    if (assuming != null) {
      var assumingValidation = AssumingSection$Companion_getInstance().validate_rk66c5$(assuming);
      if (assumingValidation.isSuccessful) {
        assumingSection = ensureNotNull(assumingValidation.value);
      }
       else {
        errors.addAll_brywnq$(assumingValidation.errors);
      }
    }
    var endValidation = validateEndSection(ensureNotNull(end));
    var endSection = null;
    if (endValidation.isSuccessful) {
      endSection = endValidation.value;
    }
     else {
      errors.addAll_brywnq$(endValidation.errors);
    }
    var aliasSection = null;
    if (alias != null) {
      var aliasValidation = AliasSection$Companion_getInstance().validate_3fjnpj$(alias);
      if (aliasValidation.isSuccessful) {
        aliasSection = ensureNotNull(aliasValidation.value);
      }
       else {
        errors.addAll_brywnq$(aliasValidation.errors);
      }
    }
    var metaDataSection = null;
    if (metadata != null) {
      var metaDataValidation = MetaDataSection$Companion_getInstance().validate_3fjnpj$(metadata);
      if (metaDataValidation.isSuccessful) {
        metaDataSection = ensureNotNull(metaDataValidation.value);
      }
       else {
        errors.addAll_brywnq$(metaDataValidation.errors);
      }
    }
    if (!errors.isEmpty()) {
      tmp$_0 = Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
     else
      tmp$_0 = Validation$Companion_getInstance().success_mh5how$(buildGroup(getSignature(ensureNotNull(id)), id, ensureNotNull(definesLikeSection), assumingSection, ensureNotNull(endSection), aliasSection, metaDataSection));
    return tmp$_0;
  }
  function getOrNull($receiver, key) {
    return $receiver.containsKey_11rb$(key) ? $receiver.get_11rb$(key) : null;
  }
  function getSignature(stmt) {
    var tmp$;
    var sigs = findAllStatementSignatures(stmt);
    if (sigs.size === 1) {
      tmp$ = first(sigs);
    }
     else
      tmp$ = null;
    return tmp$;
  }
  function findAllStatementSignatures(stmt) {
    var rootValidation = stmt.texTalkRoot;
    if (!rootValidation.isSuccessful) {
      return emptySet();
    }
    var expressionNode = ensureNotNull(rootValidation.value);
    var signatures = LinkedHashSet_init();
    findAllSignaturesImpl_0(expressionNode, signatures);
    return signatures;
  }
  function findAllSignaturesImpl$lambda_0(closure$signatures) {
    return function (it) {
      findAllSignaturesImpl_0(it, closure$signatures);
      return Unit;
    };
  }
  function findAllSignaturesImpl_0(node, signatures) {
    var tmp$;
    if (Kotlin.isType(node, IsNode)) {
      tmp$ = node.rhs.items.iterator();
      while (tmp$.hasNext()) {
        var expNode = tmp$.next();
        var sig = getMergedCommandSignature(expNode);
        if (sig != null) {
          signatures.add_11rb$(sig);
        }
      }
      return;
    }
     else if (Kotlin.isType(node, Command)) {
      var sig_0 = getCommandSignature(node).toCode();
      signatures.add_11rb$(sig_0);
    }
    node.forEach_r5tgu3$(findAllSignaturesImpl$lambda_0(signatures));
  }
  function getMergedCommandSignature(expressionNode) {
    var tmp$;
    var commandParts = ArrayList_init();
    tmp$ = expressionNode.children.iterator();
    while (tmp$.hasNext()) {
      var child = tmp$.next();
      if (Kotlin.isType(child, Command)) {
        commandParts.addAll_brywnq$(child.parts);
      }
    }
    if (!commandParts.isEmpty()) {
      return getCommandSignature(new Command(commandParts)).toCode();
    }
    return null;
  }
  function getCommandSignature(command) {
    var $receiver = command.parts;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      destination.add_11rb$(getCommandPartForSignature(item));
    }
    return new Command(destination);
  }
  function callOrNull(input, fn) {
    return input == null ? null : fn(input);
  }
  function getCommandPartForSignature(node) {
    var tmp$ = node.name;
    var tmp$_0 = callOrNull(node.square, getCallableRef('getGroupNodeForSignature', function (node) {
      return getGroupNodeForSignature(node);
    }));
    var tmp$_1 = callOrNull(node.subSup, getCallableRef('getSubSupForSignature', function (node) {
      return getSubSupForSignature(node);
    }));
    var $receiver = node.groups;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$_2;
    tmp$_2 = $receiver.iterator();
    while (tmp$_2.hasNext()) {
      var item = tmp$_2.next();
      destination.add_11rb$(getGroupNodeForSignature(item));
    }
    var $receiver_0 = node.namedGroups;
    var destination_0 = ArrayList_init_0(collectionSizeOrDefault($receiver_0, 10));
    var tmp$_3;
    tmp$_3 = $receiver_0.iterator();
    while (tmp$_3.hasNext()) {
      var item_0 = tmp$_3.next();
      destination_0.add_11rb$(getNamedGroupNodeForSignature(item_0));
    }
    return new CommandPart(tmp$, tmp$_0, tmp$_1, destination, destination_0);
  }
  function getSubSupForSignature(node) {
    return new SubSupNode(callOrNull(node.sub, getCallableRef('getGroupNodeForSignature', function (node) {
      return getGroupNodeForSignature(node);
    })), callOrNull(node.sup, getCallableRef('getGroupNodeForSignature', function (node) {
      return getGroupNodeForSignature(node);
    })));
  }
  function getGroupNodeForSignature(node) {
    return new GroupNode(node.type, getParametersNodeForSignature(node.parameters));
  }
  function getParametersNodeForSignature(node) {
    var $receiver = node.items;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      destination.add_11rb$(new ExpressionNode(listOf(new TextNode(NodeType$Identifier_getInstance(), '?'))));
    }
    return new ParametersNode(destination);
  }
  function getNamedGroupNodeForSignature(node) {
    return new NamedGroupNode(node.name, getGroupNodeForSignature(node.group));
  }
  function MetaDataSection(mappings) {
    MetaDataSection$Companion_getInstance();
    this.mappings = mappings;
  }
  MetaDataSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.mappings.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  MetaDataSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var tmp$;
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'Metadata:'));
    builder.append_s8itvh$(10);
    tmp$ = this.mappings.size;
    for (var i = 0; i < tmp$; i++) {
      builder.append_gw00v9$(this.mappings.get_za3lpa$(i).toCode_eltk6l$(true, indent + 2 | 0));
      if (i !== (this.mappings.size - 1 | 0)) {
        builder.append_s8itvh$(10);
      }
    }
    return builder.toString();
  };
  function MetaDataSection$Companion() {
    MetaDataSection$Companion_instance = this;
  }
  MetaDataSection$Companion.prototype.validate_3fjnpj$ = function (section) {
    var tmp$, tmp$_0;
    if (!equals(section.name.text, 'Metadata')) {
      return Validation$Companion_getInstance().failure_rg4ulb$(listOf(new ParseError("Expected a 'Metadata' but found '" + section.name.text + "'", AstUtils_getInstance().getRow_rk66c5$(section), AstUtils_getInstance().getColumn_rk66c5$(section))));
    }
    var errors = ArrayList_init();
    var mappings = ArrayList_init();
    tmp$ = section.args.iterator();
    while (tmp$.hasNext()) {
      var arg = tmp$.next();
      var validation = MappingNode$Companion_getInstance().validate_rk66c5$(arg);
      if (validation.isSuccessful) {
        mappings.add_11rb$(ensureNotNull(validation.value));
      }
       else {
        errors.addAll_brywnq$(validation.errors);
      }
    }
    if (!errors.isEmpty()) {
      tmp$_0 = Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
     else {
      tmp$_0 = Validation$Companion_getInstance().success_mh5how$(new MetaDataSection(mappings));
    }
    return tmp$_0;
  };
  MetaDataSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var MetaDataSection$Companion_instance = null;
  function MetaDataSection$Companion_getInstance() {
    if (MetaDataSection$Companion_instance === null) {
      new MetaDataSection$Companion();
    }
    return MetaDataSection$Companion_instance;
  }
  MetaDataSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'MetaDataSection',
    interfaces: [Phase2Node]
  };
  MetaDataSection.prototype.component1 = function () {
    return this.mappings;
  };
  MetaDataSection.prototype.copy_rz3npo$ = function (mappings) {
    return new MetaDataSection(mappings === void 0 ? this.mappings : mappings);
  };
  MetaDataSection.prototype.toString = function () {
    return 'MetaDataSection(mappings=' + Kotlin.toString(this.mappings) + ')';
  };
  MetaDataSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.mappings) | 0;
    return result;
  };
  MetaDataSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.mappings, other.mappings))));
  };
  function appendClauseArgs(builder, clauses, indent) {
    var tmp$;
    tmp$ = clauses.size;
    for (var i = 0; i < tmp$; i++) {
      builder.append_gw00v9$(clauses.get_za3lpa$(i).toCode_eltk6l$(true, indent));
      if (i !== (clauses.size - 1 | 0)) {
        builder.append_s8itvh$(10);
      }
    }
  }
  function appendTargetArgs(builder, targets, indent) {
    var tmp$;
    tmp$ = targets.size;
    for (var i = 0; i < tmp$; i++) {
      builder.append_gw00v9$(targets.get_za3lpa$(i).toCode_eltk6l$(true, indent));
      if (i !== (targets.size - 1 | 0)) {
        builder.append_s8itvh$(10);
      }
    }
  }
  function AssumingSection(clauses) {
    AssumingSection$Companion_getInstance();
    this.clauses = clauses;
  }
  AssumingSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.clauses.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  function AssumingSection$Companion() {
    AssumingSection$Companion_instance = this;
  }
  function AssumingSection$Companion$validate$lambda(it) {
    return new AssumingSection(it);
  }
  AssumingSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return ClauseListValidator_getInstance().validate_ro44ti$(node, 'assuming', AssumingSection$Companion$validate$lambda);
  };
  AssumingSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var AssumingSection$Companion_instance = null;
  function AssumingSection$Companion_getInstance() {
    if (AssumingSection$Companion_instance === null) {
      new AssumingSection$Companion();
    }
    return AssumingSection$Companion_instance;
  }
  AssumingSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'assuming:'));
    builder.append_s8itvh$(10);
    appendClauseArgs(builder, this.clauses, indent + 2 | 0);
    return builder.toString();
  };
  AssumingSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'AssumingSection',
    interfaces: [Phase2Node]
  };
  AssumingSection.prototype.component1 = function () {
    return this.clauses;
  };
  AssumingSection.prototype.copy_9lvukv$ = function (clauses) {
    return new AssumingSection(clauses === void 0 ? this.clauses : clauses);
  };
  AssumingSection.prototype.toString = function () {
    return 'AssumingSection(clauses=' + Kotlin.toString(this.clauses) + ')';
  };
  AssumingSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.clauses) | 0;
    return result;
  };
  AssumingSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.clauses, other.clauses))));
  };
  function DefinesSection(targets) {
    DefinesSection$Companion_getInstance();
    this.targets = targets;
  }
  DefinesSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.targets.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  DefinesSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'Defines:'));
    builder.append_s8itvh$(10);
    appendTargetArgs(builder, this.targets, indent + 2 | 0);
    return builder.toString();
  };
  function DefinesSection$Companion() {
    DefinesSection$Companion_instance = this;
  }
  function DefinesSection$Companion$validate$lambda(it) {
    return new DefinesSection(it);
  }
  DefinesSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return TargetListValidator_getInstance().validate_5dzuv8$(node, 'Defines', DefinesSection$Companion$validate$lambda);
  };
  DefinesSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var DefinesSection$Companion_instance = null;
  function DefinesSection$Companion_getInstance() {
    if (DefinesSection$Companion_instance === null) {
      new DefinesSection$Companion();
    }
    return DefinesSection$Companion_instance;
  }
  DefinesSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'DefinesSection',
    interfaces: [Phase2Node]
  };
  DefinesSection.prototype.component1 = function () {
    return this.targets;
  };
  DefinesSection.prototype.copy_lv4973$ = function (targets) {
    return new DefinesSection(targets === void 0 ? this.targets : targets);
  };
  DefinesSection.prototype.toString = function () {
    return 'DefinesSection(targets=' + Kotlin.toString(this.targets) + ')';
  };
  DefinesSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.targets) | 0;
    return result;
  };
  DefinesSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.targets, other.targets))));
  };
  function RefinesSection(targets) {
    RefinesSection$Companion_getInstance();
    this.targets = targets;
  }
  RefinesSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.targets.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  RefinesSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'Refines:'));
    builder.append_s8itvh$(10);
    appendTargetArgs(builder, this.targets, indent + 2 | 0);
    return builder.toString();
  };
  function RefinesSection$Companion() {
    RefinesSection$Companion_instance = this;
  }
  function RefinesSection$Companion$validate$lambda(it) {
    return new RefinesSection(it);
  }
  RefinesSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return TargetListValidator_getInstance().validate_5dzuv8$(node, 'Refines', RefinesSection$Companion$validate$lambda);
  };
  RefinesSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var RefinesSection$Companion_instance = null;
  function RefinesSection$Companion_getInstance() {
    if (RefinesSection$Companion_instance === null) {
      new RefinesSection$Companion();
    }
    return RefinesSection$Companion_instance;
  }
  RefinesSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'RefinesSection',
    interfaces: [Phase2Node]
  };
  RefinesSection.prototype.component1 = function () {
    return this.targets;
  };
  RefinesSection.prototype.copy_lv4973$ = function (targets) {
    return new RefinesSection(targets === void 0 ? this.targets : targets);
  };
  RefinesSection.prototype.toString = function () {
    return 'RefinesSection(targets=' + Kotlin.toString(this.targets) + ')';
  };
  RefinesSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.targets) | 0;
    return result;
  };
  RefinesSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.targets, other.targets))));
  };
  function RepresentsSection() {
    RepresentsSection$Companion_getInstance();
  }
  RepresentsSection.prototype.forEach_ye21ev$ = function (fn) {
  };
  RepresentsSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return indentedString(isArg, indent, 'Represents:');
  };
  function RepresentsSection$Companion() {
    RepresentsSection$Companion_instance = this;
  }
  RepresentsSection$Companion.prototype.validate_rk66c5$ = function (node) {
    var tmp$, tmp$_0;
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Section)) {
      errors.add_11rb$(new ParseError('Expected a RepresentsSection', AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
    }
    var sect = Kotlin.isType(tmp$ = node, Section) ? tmp$ : throwCCE();
    if (!sect.args.isEmpty()) {
      errors.add_11rb$(new ParseError('A Represents cannot have any arguments', AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
    }
    if (!equals(sect.name.text, 'Represents')) {
      errors.add_11rb$(new ParseError('Expected a section named Represents', AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
    }
    if (!errors.isEmpty()) {
      tmp$_0 = Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
     else {
      tmp$_0 = Validation$Companion_getInstance().success_mh5how$(new RepresentsSection());
    }
    return tmp$_0;
  };
  RepresentsSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var RepresentsSection$Companion_instance = null;
  function RepresentsSection$Companion_getInstance() {
    if (RepresentsSection$Companion_instance === null) {
      new RepresentsSection$Companion();
    }
    return RepresentsSection$Companion_instance;
  }
  RepresentsSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'RepresentsSection',
    interfaces: [Phase2Node]
  };
  function ExistsSection(identifiers) {
    ExistsSection$Companion_getInstance();
    this.identifiers = identifiers;
  }
  ExistsSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.identifiers.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  ExistsSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'exists:'));
    builder.append_s8itvh$(10);
    appendTargetArgs(builder, this.identifiers, indent + 2 | 0);
    return builder.toString();
  };
  function ExistsSection$Companion() {
    ExistsSection$Companion_instance = this;
  }
  function ExistsSection$Companion$validate$lambda(it) {
    return new ExistsSection(it);
  }
  ExistsSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return TargetListValidator_getInstance().validate_5dzuv8$(node, 'exists', ExistsSection$Companion$validate$lambda);
  };
  ExistsSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var ExistsSection$Companion_instance = null;
  function ExistsSection$Companion_getInstance() {
    if (ExistsSection$Companion_instance === null) {
      new ExistsSection$Companion();
    }
    return ExistsSection$Companion_instance;
  }
  ExistsSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ExistsSection',
    interfaces: [Phase2Node]
  };
  ExistsSection.prototype.component1 = function () {
    return this.identifiers;
  };
  ExistsSection.prototype.copy_lv4973$ = function (identifiers) {
    return new ExistsSection(identifiers === void 0 ? this.identifiers : identifiers);
  };
  ExistsSection.prototype.toString = function () {
    return 'ExistsSection(identifiers=' + Kotlin.toString(this.identifiers) + ')';
  };
  ExistsSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.identifiers) | 0;
    return result;
  };
  ExistsSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.identifiers, other.identifiers))));
  };
  function ForSection(targets) {
    ForSection$Companion_getInstance();
    this.targets = targets;
  }
  ForSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.targets.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  ForSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'for:'));
    builder.append_s8itvh$(10);
    appendTargetArgs(builder, this.targets, indent + 2 | 0);
    return builder.toString();
  };
  function ForSection$Companion() {
    ForSection$Companion_instance = this;
  }
  function ForSection$Companion$validate$lambda(it) {
    return new ForSection(it);
  }
  ForSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return TargetListValidator_getInstance().validate_5dzuv8$(node, 'for', ForSection$Companion$validate$lambda);
  };
  ForSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var ForSection$Companion_instance = null;
  function ForSection$Companion_getInstance() {
    if (ForSection$Companion_instance === null) {
      new ForSection$Companion();
    }
    return ForSection$Companion_instance;
  }
  ForSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ForSection',
    interfaces: [Phase2Node]
  };
  ForSection.prototype.component1 = function () {
    return this.targets;
  };
  ForSection.prototype.copy_lv4973$ = function (targets) {
    return new ForSection(targets === void 0 ? this.targets : targets);
  };
  ForSection.prototype.toString = function () {
    return 'ForSection(targets=' + Kotlin.toString(this.targets) + ')';
  };
  ForSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.targets) | 0;
    return result;
  };
  ForSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.targets, other.targets))));
  };
  function MeansSection(clauses) {
    MeansSection$Companion_getInstance();
    this.clauses = clauses;
  }
  MeansSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.clauses.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  MeansSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'means:'));
    builder.append_s8itvh$(10);
    appendClauseArgs(builder, this.clauses, indent + 2 | 0);
    return builder.toString();
  };
  function MeansSection$Companion() {
    MeansSection$Companion_instance = this;
  }
  function MeansSection$Companion$validate$lambda(it) {
    return new MeansSection(it);
  }
  MeansSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return ClauseListValidator_getInstance().validate_ro44ti$(node, 'means', MeansSection$Companion$validate$lambda);
  };
  MeansSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var MeansSection$Companion_instance = null;
  function MeansSection$Companion_getInstance() {
    if (MeansSection$Companion_instance === null) {
      new MeansSection$Companion();
    }
    return MeansSection$Companion_instance;
  }
  MeansSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'MeansSection',
    interfaces: [Phase2Node]
  };
  MeansSection.prototype.component1 = function () {
    return this.clauses;
  };
  MeansSection.prototype.copy_9lvukv$ = function (clauses) {
    return new MeansSection(clauses === void 0 ? this.clauses : clauses);
  };
  MeansSection.prototype.toString = function () {
    return 'MeansSection(clauses=' + Kotlin.toString(this.clauses) + ')';
  };
  MeansSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.clauses) | 0;
    return result;
  };
  MeansSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.clauses, other.clauses))));
  };
  function ResultSection(clauses) {
    ResultSection$Companion_getInstance();
    this.clauses = clauses;
  }
  ResultSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.clauses.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  ResultSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'Result:'));
    builder.append_s8itvh$(10);
    appendClauseArgs(builder, this.clauses, indent + 2 | 0);
    return builder.toString();
  };
  function ResultSection$Companion() {
    ResultSection$Companion_instance = this;
  }
  function ResultSection$Companion$validate$lambda(it) {
    return new ResultSection(it);
  }
  ResultSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return ClauseListValidator_getInstance().validate_ro44ti$(node, 'Result', ResultSection$Companion$validate$lambda);
  };
  ResultSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var ResultSection$Companion_instance = null;
  function ResultSection$Companion_getInstance() {
    if (ResultSection$Companion_instance === null) {
      new ResultSection$Companion();
    }
    return ResultSection$Companion_instance;
  }
  ResultSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ResultSection',
    interfaces: [Phase2Node]
  };
  ResultSection.prototype.component1 = function () {
    return this.clauses;
  };
  ResultSection.prototype.copy_9lvukv$ = function (clauses) {
    return new ResultSection(clauses === void 0 ? this.clauses : clauses);
  };
  ResultSection.prototype.toString = function () {
    return 'ResultSection(clauses=' + Kotlin.toString(this.clauses) + ')';
  };
  ResultSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.clauses) | 0;
    return result;
  };
  ResultSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.clauses, other.clauses))));
  };
  function AxiomSection(clauses) {
    AxiomSection$Companion_getInstance();
    this.clauses = clauses;
  }
  AxiomSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.clauses.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  AxiomSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'Axiom:'));
    builder.append_s8itvh$(10);
    appendClauseArgs(builder, this.clauses, indent + 2 | 0);
    return builder.toString();
  };
  function AxiomSection$Companion() {
    AxiomSection$Companion_instance = this;
  }
  function AxiomSection$Companion$validate$lambda(it) {
    return new AxiomSection(it);
  }
  AxiomSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return ClauseListValidator_getInstance().validate_ro44ti$(node, 'Axiom', AxiomSection$Companion$validate$lambda);
  };
  AxiomSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var AxiomSection$Companion_instance = null;
  function AxiomSection$Companion_getInstance() {
    if (AxiomSection$Companion_instance === null) {
      new AxiomSection$Companion();
    }
    return AxiomSection$Companion_instance;
  }
  AxiomSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'AxiomSection',
    interfaces: [Phase2Node]
  };
  AxiomSection.prototype.component1 = function () {
    return this.clauses;
  };
  AxiomSection.prototype.copy_9lvukv$ = function (clauses) {
    return new AxiomSection(clauses === void 0 ? this.clauses : clauses);
  };
  AxiomSection.prototype.toString = function () {
    return 'AxiomSection(clauses=' + Kotlin.toString(this.clauses) + ')';
  };
  AxiomSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.clauses) | 0;
    return result;
  };
  AxiomSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.clauses, other.clauses))));
  };
  function ConjectureSection(clauses) {
    ConjectureSection$Companion_getInstance();
    this.clauses = clauses;
  }
  ConjectureSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.clauses.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  ConjectureSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'Conjecture:'));
    builder.append_s8itvh$(10);
    appendClauseArgs(builder, this.clauses, indent + 2 | 0);
    return builder.toString();
  };
  function ConjectureSection$Companion() {
    ConjectureSection$Companion_instance = this;
  }
  function ConjectureSection$Companion$validate$lambda(it) {
    return new ConjectureSection(it);
  }
  ConjectureSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return ClauseListValidator_getInstance().validate_ro44ti$(node, 'Conjecture', ConjectureSection$Companion$validate$lambda);
  };
  ConjectureSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var ConjectureSection$Companion_instance = null;
  function ConjectureSection$Companion_getInstance() {
    if (ConjectureSection$Companion_instance === null) {
      new ConjectureSection$Companion();
    }
    return ConjectureSection$Companion_instance;
  }
  ConjectureSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ConjectureSection',
    interfaces: [Phase2Node]
  };
  ConjectureSection.prototype.component1 = function () {
    return this.clauses;
  };
  ConjectureSection.prototype.copy_9lvukv$ = function (clauses) {
    return new ConjectureSection(clauses === void 0 ? this.clauses : clauses);
  };
  ConjectureSection.prototype.toString = function () {
    return 'ConjectureSection(clauses=' + Kotlin.toString(this.clauses) + ')';
  };
  ConjectureSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.clauses) | 0;
    return result;
  };
  ConjectureSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.clauses, other.clauses))));
  };
  function SuchThatSection(clauses) {
    SuchThatSection$Companion_getInstance();
    this.clauses = clauses;
  }
  SuchThatSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.clauses.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  SuchThatSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'suchThat:'));
    builder.append_s8itvh$(10);
    appendClauseArgs(builder, this.clauses, indent + 2 | 0);
    return builder.toString();
  };
  function SuchThatSection$Companion() {
    SuchThatSection$Companion_instance = this;
  }
  function SuchThatSection$Companion$validate$lambda(it) {
    return new SuchThatSection(it);
  }
  SuchThatSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return ClauseListValidator_getInstance().validate_ro44ti$(node, 'suchThat', SuchThatSection$Companion$validate$lambda);
  };
  SuchThatSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var SuchThatSection$Companion_instance = null;
  function SuchThatSection$Companion_getInstance() {
    if (SuchThatSection$Companion_instance === null) {
      new SuchThatSection$Companion();
    }
    return SuchThatSection$Companion_instance;
  }
  SuchThatSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'SuchThatSection',
    interfaces: [Phase2Node]
  };
  SuchThatSection.prototype.component1 = function () {
    return this.clauses;
  };
  SuchThatSection.prototype.copy_9lvukv$ = function (clauses) {
    return new SuchThatSection(clauses === void 0 ? this.clauses : clauses);
  };
  SuchThatSection.prototype.toString = function () {
    return 'SuchThatSection(clauses=' + Kotlin.toString(this.clauses) + ')';
  };
  SuchThatSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.clauses) | 0;
    return result;
  };
  SuchThatSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.clauses, other.clauses))));
  };
  function ThatSection(clauses) {
    ThatSection$Companion_getInstance();
    this.clauses = clauses;
  }
  ThatSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.clauses.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  ThatSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'that:'));
    builder.append_s8itvh$(10);
    appendClauseArgs(builder, this.clauses, indent + 2 | 0);
    return builder.toString();
  };
  function ThatSection$Companion() {
    ThatSection$Companion_instance = this;
  }
  function ThatSection$Companion$validate$lambda(it) {
    return new ThatSection(it);
  }
  ThatSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return ClauseListValidator_getInstance().validate_ro44ti$(node, 'that', ThatSection$Companion$validate$lambda);
  };
  ThatSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var ThatSection$Companion_instance = null;
  function ThatSection$Companion_getInstance() {
    if (ThatSection$Companion_instance === null) {
      new ThatSection$Companion();
    }
    return ThatSection$Companion_instance;
  }
  ThatSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ThatSection',
    interfaces: [Phase2Node]
  };
  ThatSection.prototype.component1 = function () {
    return this.clauses;
  };
  ThatSection.prototype.copy_9lvukv$ = function (clauses) {
    return new ThatSection(clauses === void 0 ? this.clauses : clauses);
  };
  ThatSection.prototype.toString = function () {
    return 'ThatSection(clauses=' + Kotlin.toString(this.clauses) + ')';
  };
  ThatSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.clauses) | 0;
    return result;
  };
  ThatSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.clauses, other.clauses))));
  };
  function IfSection(clauses) {
    IfSection$Companion_getInstance();
    this.clauses = clauses;
  }
  IfSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.clauses.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  IfSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'if:'));
    builder.append_s8itvh$(10);
    appendClauseArgs(builder, this.clauses, indent + 2 | 0);
    return builder.toString();
  };
  function IfSection$Companion() {
    IfSection$Companion_instance = this;
  }
  function IfSection$Companion$validate$lambda(it) {
    return new IfSection(it);
  }
  IfSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return ClauseListValidator_getInstance().validate_ro44ti$(node, 'if', IfSection$Companion$validate$lambda);
  };
  IfSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var IfSection$Companion_instance = null;
  function IfSection$Companion_getInstance() {
    if (IfSection$Companion_instance === null) {
      new IfSection$Companion();
    }
    return IfSection$Companion_instance;
  }
  IfSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'IfSection',
    interfaces: [Phase2Node]
  };
  IfSection.prototype.component1 = function () {
    return this.clauses;
  };
  IfSection.prototype.copy_9lvukv$ = function (clauses) {
    return new IfSection(clauses === void 0 ? this.clauses : clauses);
  };
  IfSection.prototype.toString = function () {
    return 'IfSection(clauses=' + Kotlin.toString(this.clauses) + ')';
  };
  IfSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.clauses) | 0;
    return result;
  };
  IfSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.clauses, other.clauses))));
  };
  function IffSection(clauses) {
    IffSection$Companion_getInstance();
    this.clauses = clauses;
  }
  IffSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.clauses.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  IffSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'iff:'));
    builder.append_s8itvh$(10);
    appendClauseArgs(builder, this.clauses, indent + 2 | 0);
    return builder.toString();
  };
  function IffSection$Companion() {
    IffSection$Companion_instance = this;
  }
  function IffSection$Companion$validate$lambda(it) {
    return new IffSection(it);
  }
  IffSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return ClauseListValidator_getInstance().validate_ro44ti$(node, 'iff', IffSection$Companion$validate$lambda);
  };
  IffSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var IffSection$Companion_instance = null;
  function IffSection$Companion_getInstance() {
    if (IffSection$Companion_instance === null) {
      new IffSection$Companion();
    }
    return IffSection$Companion_instance;
  }
  IffSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'IffSection',
    interfaces: [Phase2Node]
  };
  IffSection.prototype.component1 = function () {
    return this.clauses;
  };
  IffSection.prototype.copy_9lvukv$ = function (clauses) {
    return new IffSection(clauses === void 0 ? this.clauses : clauses);
  };
  IffSection.prototype.toString = function () {
    return 'IffSection(clauses=' + Kotlin.toString(this.clauses) + ')';
  };
  IffSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.clauses) | 0;
    return result;
  };
  IffSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.clauses, other.clauses))));
  };
  function ThenSection(clauses) {
    ThenSection$Companion_getInstance();
    this.clauses = clauses;
  }
  ThenSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.clauses.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  ThenSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'then:'));
    builder.append_s8itvh$(10);
    appendClauseArgs(builder, this.clauses, indent + 2 | 0);
    return builder.toString();
  };
  function ThenSection$Companion() {
    ThenSection$Companion_instance = this;
  }
  function ThenSection$Companion$validate$lambda(it) {
    return new ThenSection(it);
  }
  ThenSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return ClauseListValidator_getInstance().validate_ro44ti$(node, 'then', ThenSection$Companion$validate$lambda);
  };
  ThenSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var ThenSection$Companion_instance = null;
  function ThenSection$Companion_getInstance() {
    if (ThenSection$Companion_instance === null) {
      new ThenSection$Companion();
    }
    return ThenSection$Companion_instance;
  }
  ThenSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ThenSection',
    interfaces: [Phase2Node]
  };
  ThenSection.prototype.component1 = function () {
    return this.clauses;
  };
  ThenSection.prototype.copy_9lvukv$ = function (clauses) {
    return new ThenSection(clauses === void 0 ? this.clauses : clauses);
  };
  ThenSection.prototype.toString = function () {
    return 'ThenSection(clauses=' + Kotlin.toString(this.clauses) + ')';
  };
  ThenSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.clauses) | 0;
    return result;
  };
  ThenSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.clauses, other.clauses))));
  };
  function WhereSection(clauses) {
    WhereSection$Companion_getInstance();
    this.clauses = clauses;
  }
  WhereSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.clauses.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  WhereSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'where:'));
    builder.append_s8itvh$(10);
    appendClauseArgs(builder, this.clauses, indent + 2 | 0);
    return builder.toString();
  };
  function WhereSection$Companion() {
    WhereSection$Companion_instance = this;
  }
  function WhereSection$Companion$validate$lambda(it) {
    return new WhereSection(it);
  }
  WhereSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return ClauseListValidator_getInstance().validate_ro44ti$(node, 'where', WhereSection$Companion$validate$lambda);
  };
  WhereSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var WhereSection$Companion_instance = null;
  function WhereSection$Companion_getInstance() {
    if (WhereSection$Companion_instance === null) {
      new WhereSection$Companion();
    }
    return WhereSection$Companion_instance;
  }
  WhereSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'WhereSection',
    interfaces: [Phase2Node]
  };
  WhereSection.prototype.component1 = function () {
    return this.clauses;
  };
  WhereSection.prototype.copy_9lvukv$ = function (clauses) {
    return new WhereSection(clauses === void 0 ? this.clauses : clauses);
  };
  WhereSection.prototype.toString = function () {
    return 'WhereSection(clauses=' + Kotlin.toString(this.clauses) + ')';
  };
  WhereSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.clauses) | 0;
    return result;
  };
  WhereSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.clauses, other.clauses))));
  };
  function NotSection(clauses) {
    NotSection$Companion_getInstance();
    this.clauses = clauses;
  }
  NotSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.clauses.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  NotSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'not:'));
    builder.append_s8itvh$(10);
    appendClauseArgs(builder, this.clauses, indent + 2 | 0);
    return builder.toString();
  };
  function NotSection$Companion() {
    NotSection$Companion_instance = this;
  }
  function NotSection$Companion$validate$lambda(it) {
    return new NotSection(it);
  }
  NotSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return ClauseListValidator_getInstance().validate_ro44ti$(node, 'not', NotSection$Companion$validate$lambda);
  };
  NotSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var NotSection$Companion_instance = null;
  function NotSection$Companion_getInstance() {
    if (NotSection$Companion_instance === null) {
      new NotSection$Companion();
    }
    return NotSection$Companion_instance;
  }
  NotSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'NotSection',
    interfaces: [Phase2Node]
  };
  NotSection.prototype.component1 = function () {
    return this.clauses;
  };
  NotSection.prototype.copy_9lvukv$ = function (clauses) {
    return new NotSection(clauses === void 0 ? this.clauses : clauses);
  };
  NotSection.prototype.toString = function () {
    return 'NotSection(clauses=' + Kotlin.toString(this.clauses) + ')';
  };
  NotSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.clauses) | 0;
    return result;
  };
  NotSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.clauses, other.clauses))));
  };
  function OrSection(clauses) {
    OrSection$Companion_getInstance();
    this.clauses = clauses;
  }
  OrSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.clauses.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  OrSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'or:'));
    builder.append_s8itvh$(10);
    appendClauseArgs(builder, this.clauses, indent + 2 | 0);
    return builder.toString();
  };
  function OrSection$Companion() {
    OrSection$Companion_instance = this;
  }
  function OrSection$Companion$validate$lambda(it) {
    return new OrSection(it);
  }
  OrSection$Companion.prototype.validate_rk66c5$ = function (node) {
    return ClauseListValidator_getInstance().validate_ro44ti$(node, 'or', OrSection$Companion$validate$lambda);
  };
  OrSection$Companion.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'Companion',
    interfaces: []
  };
  var OrSection$Companion_instance = null;
  function OrSection$Companion_getInstance() {
    if (OrSection$Companion_instance === null) {
      new OrSection$Companion();
    }
    return OrSection$Companion_instance;
  }
  OrSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'OrSection',
    interfaces: [Phase2Node]
  };
  OrSection.prototype.component1 = function () {
    return this.clauses;
  };
  OrSection.prototype.copy_9lvukv$ = function (clauses) {
    return new OrSection(clauses === void 0 ? this.clauses : clauses);
  };
  OrSection.prototype.toString = function () {
    return 'OrSection(clauses=' + Kotlin.toString(this.clauses) + ')';
  };
  OrSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.clauses) | 0;
    return result;
  };
  OrSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.clauses, other.clauses))));
  };
  function SectionIdentifier() {
    SectionIdentifier_instance = this;
  }
  SectionIdentifier.prototype.identifySections_b3nzct$ = function (sections, expected) {
    var tmp$, tmp$_0, tmp$_1, tmp$_2;
    var patternBuilder = StringBuilder_init();
    for (tmp$ = 0; tmp$ !== expected.length; ++tmp$) {
      var name = expected[tmp$];
      patternBuilder.append_gw00v9$(name);
      patternBuilder.append_gw00v9$(':\n');
    }
    var pattern = patternBuilder.toString();
    var sectionQueue = new Queue();
    tmp$_0 = sections.iterator();
    while (tmp$_0.hasNext()) {
      var s = tmp$_0.next();
      sectionQueue.offer_11rb$(s);
    }
    var expectedQueue = new Queue();
    for (tmp$_1 = 0; tmp$_1 !== expected.length; ++tmp$_1) {
      var e = expected[tmp$_1];
      expectedQueue.offer_11rb$(e);
    }
    var result = HashMap_init();
    while (!sectionQueue.isEmpty() && !expectedQueue.isEmpty()) {
      var nextSection = sectionQueue.peek();
      var maybeName = expectedQueue.peek();
      var isOptional = endsWith(maybeName, '?');
      var tmp$_3;
      if (isOptional) {
        var endIndex = maybeName.length - 1 | 0;
        tmp$_3 = maybeName.substring(0, endIndex);
      }
       else
        tmp$_3 = maybeName;
      var trueName = tmp$_3;
      if (equals(nextSection.name.text, trueName)) {
        result.put_xwzc9p$(trueName, nextSection);
        sectionQueue.poll();
        expectedQueue.poll();
      }
       else if (isOptional) {
        expectedQueue.poll();
      }
       else {
        throw new ParseError('For pattern:\n\n' + pattern + "\nExpected '" + trueName + "' but found '" + nextSection.name.text + "'", AstUtils_getInstance().getRow_rk66c5$(nextSection), AstUtils_getInstance().getColumn_rk66c5$(nextSection));
      }
    }
    if (!sectionQueue.isEmpty()) {
      var peek = sectionQueue.peek();
      throw new ParseError('For pattern:\n\n' + pattern + "\nUnexpected Section '" + peek.name.text + "'", AstUtils_getInstance().getRow_rk66c5$(peek), AstUtils_getInstance().getColumn_rk66c5$(peek));
    }
    var nextExpected = null;
    tmp$_2 = expectedQueue.iterator();
    while (tmp$_2.hasNext()) {
      var exp = tmp$_2.next();
      if (!endsWith(exp, '?')) {
        nextExpected = exp;
        break;
      }
    }
    var startRow = -1;
    var startColumn = -1;
    if (!sections.isEmpty()) {
      var sect = sections.get_za3lpa$(0);
      startRow = AstUtils_getInstance().getRow_rk66c5$(sect);
      startColumn = AstUtils_getInstance().getColumn_rk66c5$(sect);
    }
    if (nextExpected != null) {
      throw new ParseError('For pattern:\n\n' + pattern + '\nExpected a ' + nextExpected, startRow, startColumn);
    }
    return result;
  };
  SectionIdentifier.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'SectionIdentifier',
    interfaces: []
  };
  var SectionIdentifier_instance = null;
  function SectionIdentifier_getInstance() {
    if (SectionIdentifier_instance === null) {
      new SectionIdentifier();
    }
    return SectionIdentifier_instance;
  }
  function Queue() {
    this.data_0 = ArrayList_init();
  }
  Queue.prototype.offer_11rb$ = function (item) {
    this.data_0.add_wxm5ur$(0, item);
  };
  Queue.prototype.poll = function () {
    return this.data_0.removeAt_za3lpa$(0);
  };
  Queue.prototype.peek = function () {
    return this.data_0.get_za3lpa$(0);
  };
  Queue.prototype.isEmpty = function () {
    return this.data_0.isEmpty();
  };
  Queue.prototype.iterator = function () {
    return this.data_0.iterator();
  };
  Queue.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Queue',
    interfaces: [Iterable]
  };
  function TargetListSection(targets) {
    this.targets = targets;
  }
  TargetListSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'TargetListSection',
    interfaces: []
  };
  TargetListSection.prototype.component1 = function () {
    return this.targets;
  };
  TargetListSection.prototype.copy_lv4973$ = function (targets) {
    return new TargetListSection(targets === void 0 ? this.targets : targets);
  };
  TargetListSection.prototype.toString = function () {
    return 'TargetListSection(targets=' + Kotlin.toString(this.targets) + ')';
  };
  TargetListSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.targets) | 0;
    return result;
  };
  TargetListSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.targets, other.targets))));
  };
  function TargetListValidator() {
    TargetListValidator_instance = this;
  }
  TargetListValidator.prototype.validate_5dzuv8$ = function (rawNode, expectedName, builder) {
    var node = rawNode.resolve();
    var validation = this.validate_0(node, expectedName);
    if (!validation.isSuccessful) {
      return Validation$Companion_getInstance().failure_rg4ulb$(validation.errors);
    }
    var targets = ensureNotNull(validation.value).targets;
    return Validation$Companion_getInstance().success_mh5how$(builder(targets));
  };
  TargetListValidator.prototype.validate_0 = function (node, expectedName) {
    var tmp$, tmp$_0, tmp$_1;
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Section)) {
      errors.add_11rb$(new ParseError('Expected a Section', AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
    }
    var tmp$_2 = Kotlin.isType(tmp$ = node, Section) ? tmp$ : throwCCE();
    var name1 = tmp$_2.component1()
    , args = tmp$_2.component2();
    var name = name1.text;
    if (!equals(name, expectedName)) {
      errors.add_11rb$(new ParseError('Expected a Section with name ' + expectedName + ' but found ' + name, AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
    }
    var targets = ArrayList_init();
    if (args.isEmpty()) {
      errors.add_11rb$(new ParseError("Section '" + name1.text + "' requires at least one argument.", AstUtils_getInstance().getRow_rk66c5$(node), AstUtils_getInstance().getColumn_rk66c5$(node)));
    }
    tmp$_0 = args.iterator();
    while (tmp$_0.hasNext()) {
      var arg = tmp$_0.next();
      var clauseValidation = Clause$Companion_getInstance().validate_rk66c5$(arg);
      if (clauseValidation.isSuccessful) {
        var clause = clauseValidation.value;
        if (Kotlin.isType(clause, Target)) {
          targets.add_11rb$(clause);
          continue;
        }
      }
       else {
        errors.addAll_brywnq$(clauseValidation.errors);
      }
      errors.add_11rb$(new ParseError('Expected an Target', AstUtils_getInstance().getRow_rk66c5$(arg), AstUtils_getInstance().getColumn_rk66c5$(arg)));
    }
    if (!errors.isEmpty()) {
      tmp$_1 = Validation$Companion_getInstance().failure_rg4ulb$(errors);
    }
     else
      tmp$_1 = Validation$Companion_getInstance().success_mh5how$(new TargetListSection(targets));
    return tmp$_1;
  };
  TargetListValidator.$metadata$ = {
    kind: Kind_OBJECT,
    simpleName: 'TargetListValidator',
    interfaces: []
  };
  var TargetListValidator_instance = null;
  function TargetListValidator_getInstance() {
    if (TargetListValidator_instance === null) {
      new TargetListValidator();
    }
    return TargetListValidator_instance;
  }
  function NodeType(name, ordinal) {
    Enum.call(this);
    this.name$ = name;
    this.ordinal$ = ordinal;
  }
  function NodeType_initFields() {
    NodeType_initFields = function () {
    };
    NodeType$Token_instance = new NodeType('Token', 0);
    NodeType$Identifier_instance = new NodeType('Identifier', 1);
    NodeType$Operator_instance = new NodeType('Operator', 2);
    NodeType$ParenGroup_instance = new NodeType('ParenGroup', 3);
    NodeType$SquareGroup_instance = new NodeType('SquareGroup', 4);
    NodeType$CurlyGroup_instance = new NodeType('CurlyGroup', 5);
    NodeType$NamedGroup_instance = new NodeType('NamedGroup', 6);
    NodeType$Command_instance = new NodeType('Command', 7);
    NodeType$CommandPart_instance = new NodeType('CommandPart', 8);
    NodeType$Expression_instance = new NodeType('Expression', 9);
    NodeType$SubSup_instance = new NodeType('SubSup', 10);
    NodeType$Parameters_instance = new NodeType('Parameters', 11);
    NodeType$Comma_instance = new NodeType('Comma', 12);
    NodeType$Is_instance = new NodeType('Is', 13);
    NodeType$ColonEquals_instance = new NodeType('ColonEquals', 14);
  }
  var NodeType$Token_instance;
  function NodeType$Token_getInstance() {
    NodeType_initFields();
    return NodeType$Token_instance;
  }
  var NodeType$Identifier_instance;
  function NodeType$Identifier_getInstance() {
    NodeType_initFields();
    return NodeType$Identifier_instance;
  }
  var NodeType$Operator_instance;
  function NodeType$Operator_getInstance() {
    NodeType_initFields();
    return NodeType$Operator_instance;
  }
  var NodeType$ParenGroup_instance;
  function NodeType$ParenGroup_getInstance() {
    NodeType_initFields();
    return NodeType$ParenGroup_instance;
  }
  var NodeType$SquareGroup_instance;
  function NodeType$SquareGroup_getInstance() {
    NodeType_initFields();
    return NodeType$SquareGroup_instance;
  }
  var NodeType$CurlyGroup_instance;
  function NodeType$CurlyGroup_getInstance() {
    NodeType_initFields();
    return NodeType$CurlyGroup_instance;
  }
  var NodeType$NamedGroup_instance;
  function NodeType$NamedGroup_getInstance() {
    NodeType_initFields();
    return NodeType$NamedGroup_instance;
  }
  var NodeType$Command_instance;
  function NodeType$Command_getInstance() {
    NodeType_initFields();
    return NodeType$Command_instance;
  }
  var NodeType$CommandPart_instance;
  function NodeType$CommandPart_getInstance() {
    NodeType_initFields();
    return NodeType$CommandPart_instance;
  }
  var NodeType$Expression_instance;
  function NodeType$Expression_getInstance() {
    NodeType_initFields();
    return NodeType$Expression_instance;
  }
  var NodeType$SubSup_instance;
  function NodeType$SubSup_getInstance() {
    NodeType_initFields();
    return NodeType$SubSup_instance;
  }
  var NodeType$Parameters_instance;
  function NodeType$Parameters_getInstance() {
    NodeType_initFields();
    return NodeType$Parameters_instance;
  }
  var NodeType$Comma_instance;
  function NodeType$Comma_getInstance() {
    NodeType_initFields();
    return NodeType$Comma_instance;
  }
  var NodeType$Is_instance;
  function NodeType$Is_getInstance() {
    NodeType_initFields();
    return NodeType$Is_instance;
  }
  var NodeType$ColonEquals_instance;
  function NodeType$ColonEquals_getInstance() {
    NodeType_initFields();
    return NodeType$ColonEquals_instance;
  }
  NodeType.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'NodeType',
    interfaces: [Enum]
  };
  function NodeType$values() {
    return [NodeType$Token_getInstance(), NodeType$Identifier_getInstance(), NodeType$Operator_getInstance(), NodeType$ParenGroup_getInstance(), NodeType$SquareGroup_getInstance(), NodeType$CurlyGroup_getInstance(), NodeType$NamedGroup_getInstance(), NodeType$Command_getInstance(), NodeType$CommandPart_getInstance(), NodeType$Expression_getInstance(), NodeType$SubSup_getInstance(), NodeType$Parameters_getInstance(), NodeType$Comma_getInstance(), NodeType$Is_getInstance(), NodeType$ColonEquals_getInstance()];
  }
  NodeType.values = NodeType$values;
  function NodeType$valueOf(name) {
    switch (name) {
      case 'Token':
        return NodeType$Token_getInstance();
      case 'Identifier':
        return NodeType$Identifier_getInstance();
      case 'Operator':
        return NodeType$Operator_getInstance();
      case 'ParenGroup':
        return NodeType$ParenGroup_getInstance();
      case 'SquareGroup':
        return NodeType$SquareGroup_getInstance();
      case 'CurlyGroup':
        return NodeType$CurlyGroup_getInstance();
      case 'NamedGroup':
        return NodeType$NamedGroup_getInstance();
      case 'Command':
        return NodeType$Command_getInstance();
      case 'CommandPart':
        return NodeType$CommandPart_getInstance();
      case 'Expression':
        return NodeType$Expression_getInstance();
      case 'SubSup':
        return NodeType$SubSup_getInstance();
      case 'Parameters':
        return NodeType$Parameters_getInstance();
      case 'Comma':
        return NodeType$Comma_getInstance();
      case 'Is':
        return NodeType$Is_getInstance();
      case 'ColonEquals':
        return NodeType$ColonEquals_getInstance();
      default:throwISE('No enum constant mathlingua.common.textalk.NodeType.' + name);
    }
  }
  NodeType.valueOf_61zpoe$ = NodeType$valueOf;
  function Node() {
  }
  Node.$metadata$ = {
    kind: Kind_INTERFACE,
    simpleName: 'Node',
    interfaces: []
  };
  function IsNode(lhs, rhs) {
    this.lhs = lhs;
    this.rhs = rhs;
  }
  Object.defineProperty(IsNode.prototype, 'type', {
    get: function () {
      return NodeType$Is_getInstance();
    }
  });
  IsNode.prototype.toCode = function () {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(this.lhs.toCode());
    builder.append_gw00v9$(' is ');
    builder.append_gw00v9$(this.rhs.toCode());
    return builder.toString();
  };
  IsNode.prototype.forEach_r5tgu3$ = function (fn) {
    fn(this.lhs);
    fn(this.rhs);
  };
  IsNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'IsNode',
    interfaces: [Node]
  };
  IsNode.prototype.component1 = function () {
    return this.lhs;
  };
  IsNode.prototype.component2 = function () {
    return this.rhs;
  };
  IsNode.prototype.copy_g7o2dw$ = function (lhs, rhs) {
    return new IsNode(lhs === void 0 ? this.lhs : lhs, rhs === void 0 ? this.rhs : rhs);
  };
  IsNode.prototype.toString = function () {
    return 'IsNode(lhs=' + Kotlin.toString(this.lhs) + (', rhs=' + Kotlin.toString(this.rhs)) + ')';
  };
  IsNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.lhs) | 0;
    result = result * 31 + Kotlin.hashCode(this.rhs) | 0;
    return result;
  };
  IsNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.lhs, other.lhs) && Kotlin.equals(this.rhs, other.rhs)))));
  };
  function ColonEqualsNode(lhs, rhs) {
    this.lhs = lhs;
    this.rhs = rhs;
  }
  Object.defineProperty(ColonEqualsNode.prototype, 'type', {
    get: function () {
      return NodeType$ColonEquals_getInstance();
    }
  });
  ColonEqualsNode.prototype.toCode = function () {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(this.lhs.toCode());
    builder.append_gw00v9$(' := ');
    builder.append_gw00v9$(this.rhs.toCode());
    return builder.toString();
  };
  ColonEqualsNode.prototype.forEach_r5tgu3$ = function (fn) {
    fn(this.lhs);
    fn(this.rhs);
  };
  ColonEqualsNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ColonEqualsNode',
    interfaces: [Node]
  };
  ColonEqualsNode.prototype.component1 = function () {
    return this.lhs;
  };
  ColonEqualsNode.prototype.component2 = function () {
    return this.rhs;
  };
  ColonEqualsNode.prototype.copy_g7o2dw$ = function (lhs, rhs) {
    return new ColonEqualsNode(lhs === void 0 ? this.lhs : lhs, rhs === void 0 ? this.rhs : rhs);
  };
  ColonEqualsNode.prototype.toString = function () {
    return 'ColonEqualsNode(lhs=' + Kotlin.toString(this.lhs) + (', rhs=' + Kotlin.toString(this.rhs)) + ')';
  };
  ColonEqualsNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.lhs) | 0;
    result = result * 31 + Kotlin.hashCode(this.rhs) | 0;
    return result;
  };
  ColonEqualsNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.lhs, other.lhs) && Kotlin.equals(this.rhs, other.rhs)))));
  };
  function CommandPart(name, square, subSup, groups, namedGroups) {
    this.name = name;
    this.square = square;
    this.subSup = subSup;
    this.groups = groups;
    this.namedGroups = namedGroups;
  }
  Object.defineProperty(CommandPart.prototype, 'type', {
    get: function () {
      return NodeType$CommandPart_getInstance();
    }
  });
  CommandPart.prototype.toCode = function () {
    var tmp$, tmp$_0;
    var buffer = StringBuilder_init();
    buffer.append_gw00v9$(this.name.toCode());
    if (this.square != null) {
      buffer.append_gw00v9$(this.square.toCode());
    }
    if (this.subSup != null) {
      buffer.append_gw00v9$(this.subSup.toCode());
    }
    tmp$ = this.groups.iterator();
    while (tmp$.hasNext()) {
      var grp = tmp$.next();
      buffer.append_gw00v9$(grp.toCode());
    }
    if (!this.namedGroups.isEmpty()) {
      buffer.append_gw00v9$(':');
    }
    tmp$_0 = this.namedGroups.iterator();
    while (tmp$_0.hasNext()) {
      var namedGrp = tmp$_0.next();
      buffer.append_gw00v9$(namedGrp.toCode());
    }
    return buffer.toString();
  };
  CommandPart.prototype.forEach_r5tgu3$ = function (fn) {
    fn(this.name);
    if (this.square != null) {
      fn(this.square);
    }
    if (this.subSup != null) {
      fn(this.subSup);
    }
    var tmp$;
    tmp$ = this.groups.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
    var tmp$_0;
    tmp$_0 = this.namedGroups.iterator();
    while (tmp$_0.hasNext()) {
      var element_0 = tmp$_0.next();
      fn(element_0);
    }
  };
  CommandPart.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'CommandPart',
    interfaces: [Node]
  };
  CommandPart.prototype.component1 = function () {
    return this.name;
  };
  CommandPart.prototype.component2 = function () {
    return this.square;
  };
  CommandPart.prototype.component3 = function () {
    return this.subSup;
  };
  CommandPart.prototype.component4 = function () {
    return this.groups;
  };
  CommandPart.prototype.component5 = function () {
    return this.namedGroups;
  };
  CommandPart.prototype.copy_zoas3$ = function (name, square, subSup, groups, namedGroups) {
    return new CommandPart(name === void 0 ? this.name : name, square === void 0 ? this.square : square, subSup === void 0 ? this.subSup : subSup, groups === void 0 ? this.groups : groups, namedGroups === void 0 ? this.namedGroups : namedGroups);
  };
  CommandPart.prototype.toString = function () {
    return 'CommandPart(name=' + Kotlin.toString(this.name) + (', square=' + Kotlin.toString(this.square)) + (', subSup=' + Kotlin.toString(this.subSup)) + (', groups=' + Kotlin.toString(this.groups)) + (', namedGroups=' + Kotlin.toString(this.namedGroups)) + ')';
  };
  CommandPart.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.name) | 0;
    result = result * 31 + Kotlin.hashCode(this.square) | 0;
    result = result * 31 + Kotlin.hashCode(this.subSup) | 0;
    result = result * 31 + Kotlin.hashCode(this.groups) | 0;
    result = result * 31 + Kotlin.hashCode(this.namedGroups) | 0;
    return result;
  };
  CommandPart.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.name, other.name) && Kotlin.equals(this.square, other.square) && Kotlin.equals(this.subSup, other.subSup) && Kotlin.equals(this.groups, other.groups) && Kotlin.equals(this.namedGroups, other.namedGroups)))));
  };
  function Command(parts) {
    this.parts = parts;
  }
  Object.defineProperty(Command.prototype, 'type', {
    get: function () {
      return NodeType$Command_getInstance();
    }
  });
  Command.prototype.toCode = function () {
    var tmp$;
    var builder = new StringBuilder('\\');
    tmp$ = this.parts.size;
    for (var i = 0; i < tmp$; i++) {
      if (i > 0) {
        builder.append_s8itvh$(46);
      }
      builder.append_gw00v9$(this.parts.get_za3lpa$(i).toCode());
    }
    return builder.toString();
  };
  Command.prototype.forEach_r5tgu3$ = function (fn) {
    var tmp$;
    tmp$ = this.parts.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  Command.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Command',
    interfaces: [Node]
  };
  Command.prototype.component1 = function () {
    return this.parts;
  };
  Command.prototype.copy_aj4jsd$ = function (parts) {
    return new Command(parts === void 0 ? this.parts : parts);
  };
  Command.prototype.toString = function () {
    return 'Command(parts=' + Kotlin.toString(this.parts) + ')';
  };
  Command.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.parts) | 0;
    return result;
  };
  Command.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.parts, other.parts))));
  };
  function ExpressionNode(children) {
    this.children = children;
  }
  Object.defineProperty(ExpressionNode.prototype, 'type', {
    get: function () {
      return NodeType$Expression_getInstance();
    }
  });
  ExpressionNode.prototype.toCode = function () {
    var builder = StringBuilder_init();
    var children = this.children;
    for (var i = 0; i !== children.size; ++i) {
      var child = children.get_za3lpa$(i);
      builder.append_gw00v9$(child.toCode());
      if (i !== (children.size - 1 | 0)) {
        builder.append_gw00v9$(' ');
      }
    }
    return builder.toString();
  };
  ExpressionNode.prototype.forEach_r5tgu3$ = function (fn) {
    var tmp$;
    tmp$ = this.children.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  ExpressionNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ExpressionNode',
    interfaces: [Node]
  };
  ExpressionNode.prototype.component1 = function () {
    return this.children;
  };
  ExpressionNode.prototype.copy_9zq48h$ = function (children) {
    return new ExpressionNode(children === void 0 ? this.children : children);
  };
  ExpressionNode.prototype.toString = function () {
    return 'ExpressionNode(children=' + Kotlin.toString(this.children) + ')';
  };
  ExpressionNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.children) | 0;
    return result;
  };
  ExpressionNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.children, other.children))));
  };
  function ParametersNode(items) {
    this.items = items;
  }
  Object.defineProperty(ParametersNode.prototype, 'type', {
    get: function () {
      return NodeType$Parameters_getInstance();
    }
  });
  ParametersNode.prototype.toCode = function () {
    var tmp$;
    var buffer = StringBuilder_init();
    if (!this.items.isEmpty()) {
      buffer.append_gw00v9$(this.items.get_za3lpa$(0).toCode());
    }
    tmp$ = this.items.size;
    for (var i = 1; i < tmp$; i++) {
      buffer.append_gw00v9$(', ');
      buffer.append_gw00v9$(this.items.get_za3lpa$(i).toCode());
    }
    return buffer.toString();
  };
  ParametersNode.prototype.forEach_r5tgu3$ = function (fn) {
    var tmp$;
    tmp$ = this.items.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  ParametersNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ParametersNode',
    interfaces: [Node]
  };
  ParametersNode.prototype.component1 = function () {
    return this.items;
  };
  ParametersNode.prototype.copy_kt9y1j$ = function (items) {
    return new ParametersNode(items === void 0 ? this.items : items);
  };
  ParametersNode.prototype.toString = function () {
    return 'ParametersNode(items=' + Kotlin.toString(this.items) + ')';
  };
  ParametersNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.items) | 0;
    return result;
  };
  ParametersNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.items, other.items))));
  };
  function GroupNode(type, parameters) {
    this.type_9mp3jb$_0 = type;
    this.parameters = parameters;
  }
  Object.defineProperty(GroupNode.prototype, 'type', {
    get: function () {
      return this.type_9mp3jb$_0;
    }
  });
  GroupNode.prototype.toCode = function () {
    var prefix;
    var suffix;
    switch (this.type.name) {
      case 'ParenGroup':
        prefix = '(';
        suffix = ')';
        break;
      case 'SquareGroup':
        prefix = '[';
        suffix = ']';
        break;
      case 'CurlyGroup':
        prefix = '{';
        suffix = '}';
        break;
      default:throw RuntimeException_init('Unrecognized group type ' + this.type);
    }
    var buffer = new StringBuilder(prefix);
    buffer.append_gw00v9$(this.parameters.toCode());
    buffer.append_gw00v9$(suffix);
    return buffer.toString();
  };
  GroupNode.prototype.forEach_r5tgu3$ = function (fn) {
    fn(this.parameters);
  };
  GroupNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'GroupNode',
    interfaces: [Node]
  };
  GroupNode.prototype.component1 = function () {
    return this.type;
  };
  GroupNode.prototype.component2 = function () {
    return this.parameters;
  };
  GroupNode.prototype.copy_4tz8rg$ = function (type, parameters) {
    return new GroupNode(type === void 0 ? this.type : type, parameters === void 0 ? this.parameters : parameters);
  };
  GroupNode.prototype.toString = function () {
    return 'GroupNode(type=' + Kotlin.toString(this.type) + (', parameters=' + Kotlin.toString(this.parameters)) + ')';
  };
  GroupNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.type) | 0;
    result = result * 31 + Kotlin.hashCode(this.parameters) | 0;
    return result;
  };
  GroupNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.type, other.type) && Kotlin.equals(this.parameters, other.parameters)))));
  };
  function NamedGroupNode(name, group) {
    this.name = name;
    this.group = group;
  }
  Object.defineProperty(NamedGroupNode.prototype, 'type', {
    get: function () {
      return NodeType$NamedGroup_getInstance();
    }
  });
  NamedGroupNode.prototype.toCode = function () {
    var buffer = StringBuilder_init();
    buffer.append_gw00v9$(this.name.toCode());
    buffer.append_gw00v9$(this.group.toCode());
    return buffer.toString();
  };
  NamedGroupNode.prototype.forEach_r5tgu3$ = function (fn) {
    fn(this.name);
    fn(this.group);
  };
  NamedGroupNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'NamedGroupNode',
    interfaces: [Node]
  };
  NamedGroupNode.prototype.component1 = function () {
    return this.name;
  };
  NamedGroupNode.prototype.component2 = function () {
    return this.group;
  };
  NamedGroupNode.prototype.copy_9ups78$ = function (name, group) {
    return new NamedGroupNode(name === void 0 ? this.name : name, group === void 0 ? this.group : group);
  };
  NamedGroupNode.prototype.toString = function () {
    return 'NamedGroupNode(name=' + Kotlin.toString(this.name) + (', group=' + Kotlin.toString(this.group)) + ')';
  };
  NamedGroupNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.name) | 0;
    result = result * 31 + Kotlin.hashCode(this.group) | 0;
    return result;
  };
  NamedGroupNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.name, other.name) && Kotlin.equals(this.group, other.group)))));
  };
  function SubSupNode(sub, sup) {
    this.sub = sub;
    this.sup = sup;
  }
  Object.defineProperty(SubSupNode.prototype, 'type', {
    get: function () {
      return NodeType$SubSup_getInstance();
    }
  });
  SubSupNode.prototype.toCode = function () {
    var builder = StringBuilder_init();
    if (this.sub != null) {
      builder.append_gw00v9$('_');
      builder.append_gw00v9$(this.sub.toCode());
    }
    if (this.sup != null) {
      builder.append_gw00v9$('^');
      builder.append_gw00v9$(this.sup.toCode());
    }
    return builder.toString();
  };
  SubSupNode.prototype.forEach_r5tgu3$ = function (fn) {
    if (this.sub != null) {
      fn(this.sub);
    }
    if (this.sup != null) {
      fn(this.sup);
    }
  };
  SubSupNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'SubSupNode',
    interfaces: [Node]
  };
  SubSupNode.prototype.component1 = function () {
    return this.sub;
  };
  SubSupNode.prototype.component2 = function () {
    return this.sup;
  };
  SubSupNode.prototype.copy_bpikt0$ = function (sub, sup) {
    return new SubSupNode(sub === void 0 ? this.sub : sub, sup === void 0 ? this.sup : sup);
  };
  SubSupNode.prototype.toString = function () {
    return 'SubSupNode(sub=' + Kotlin.toString(this.sub) + (', sup=' + Kotlin.toString(this.sup)) + ')';
  };
  SubSupNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.sub) | 0;
    result = result * 31 + Kotlin.hashCode(this.sup) | 0;
    return result;
  };
  SubSupNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.sub, other.sub) && Kotlin.equals(this.sup, other.sup)))));
  };
  function TextNode(type, text) {
    this.type_mj84qt$_0 = type;
    this.text = text;
  }
  Object.defineProperty(TextNode.prototype, 'type', {
    get: function () {
      return this.type_mj84qt$_0;
    }
  });
  TextNode.prototype.toCode = function () {
    return this.text;
  };
  TextNode.prototype.forEach_r5tgu3$ = function (fn) {
  };
  TextNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'TextNode',
    interfaces: [Node]
  };
  TextNode.prototype.component1 = function () {
    return this.type;
  };
  TextNode.prototype.component2 = function () {
    return this.text;
  };
  TextNode.prototype.copy_wavoco$ = function (type, text) {
    return new TextNode(type === void 0 ? this.type : type, text === void 0 ? this.text : text);
  };
  TextNode.prototype.toString = function () {
    return 'TextNode(type=' + Kotlin.toString(this.type) + (', text=' + Kotlin.toString(this.text)) + ')';
  };
  TextNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.type) | 0;
    result = result * 31 + Kotlin.hashCode(this.text) | 0;
    return result;
  };
  TextNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.type, other.type) && Kotlin.equals(this.text, other.text)))));
  };
  function TexTalkToken(text, tokenType, row, column) {
    this.text = text;
    this.tokenType = tokenType;
    this.row = row;
    this.column = column;
  }
  Object.defineProperty(TexTalkToken.prototype, 'type', {
    get: function () {
      return NodeType$Token_getInstance();
    }
  });
  TexTalkToken.prototype.toCode = function () {
    return this.text;
  };
  TexTalkToken.prototype.forEach_r5tgu3$ = function (fn) {
  };
  TexTalkToken.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'TexTalkToken',
    interfaces: [Node]
  };
  TexTalkToken.prototype.component1 = function () {
    return this.text;
  };
  TexTalkToken.prototype.component2 = function () {
    return this.tokenType;
  };
  TexTalkToken.prototype.component3 = function () {
    return this.row;
  };
  TexTalkToken.prototype.component4 = function () {
    return this.column;
  };
  TexTalkToken.prototype.copy_4t5bxk$ = function (text, tokenType, row, column) {
    return new TexTalkToken(text === void 0 ? this.text : text, tokenType === void 0 ? this.tokenType : tokenType, row === void 0 ? this.row : row, column === void 0 ? this.column : column);
  };
  TexTalkToken.prototype.toString = function () {
    return 'TexTalkToken(text=' + Kotlin.toString(this.text) + (', tokenType=' + Kotlin.toString(this.tokenType)) + (', row=' + Kotlin.toString(this.row)) + (', column=' + Kotlin.toString(this.column)) + ')';
  };
  TexTalkToken.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.text) | 0;
    result = result * 31 + Kotlin.hashCode(this.tokenType) | 0;
    result = result * 31 + Kotlin.hashCode(this.row) | 0;
    result = result * 31 + Kotlin.hashCode(this.column) | 0;
    return result;
  };
  TexTalkToken.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.text, other.text) && Kotlin.equals(this.tokenType, other.tokenType) && Kotlin.equals(this.row, other.row) && Kotlin.equals(this.column, other.column)))));
  };
  function TexTalkTokenType(name, ordinal) {
    Enum.call(this);
    this.name$ = name;
    this.ordinal$ = ordinal;
  }
  function TexTalkTokenType_initFields() {
    TexTalkTokenType_initFields = function () {
    };
    TexTalkTokenType$Backslash_instance = new TexTalkTokenType('Backslash', 0);
    TexTalkTokenType$LParen_instance = new TexTalkTokenType('LParen', 1);
    TexTalkTokenType$RParen_instance = new TexTalkTokenType('RParen', 2);
    TexTalkTokenType$LSquare_instance = new TexTalkTokenType('LSquare', 3);
    TexTalkTokenType$RSquare_instance = new TexTalkTokenType('RSquare', 4);
    TexTalkTokenType$LCurly_instance = new TexTalkTokenType('LCurly', 5);
    TexTalkTokenType$RCurly_instance = new TexTalkTokenType('RCurly', 6);
    TexTalkTokenType$Operator_instance = new TexTalkTokenType('Operator', 7);
    TexTalkTokenType$Identifier_instance = new TexTalkTokenType('Identifier', 8);
    TexTalkTokenType$Comma_instance = new TexTalkTokenType('Comma', 9);
    TexTalkTokenType$Period_instance = new TexTalkTokenType('Period', 10);
    TexTalkTokenType$Colon_instance = new TexTalkTokenType('Colon', 11);
    TexTalkTokenType$Underscore_instance = new TexTalkTokenType('Underscore', 12);
    TexTalkTokenType$Caret_instance = new TexTalkTokenType('Caret', 13);
    TexTalkTokenType$ColonEquals_instance = new TexTalkTokenType('ColonEquals', 14);
    TexTalkTokenType$Is_instance = new TexTalkTokenType('Is', 15);
    TexTalkTokenType$Invalid_instance = new TexTalkTokenType('Invalid', 16);
  }
  var TexTalkTokenType$Backslash_instance;
  function TexTalkTokenType$Backslash_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$Backslash_instance;
  }
  var TexTalkTokenType$LParen_instance;
  function TexTalkTokenType$LParen_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$LParen_instance;
  }
  var TexTalkTokenType$RParen_instance;
  function TexTalkTokenType$RParen_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$RParen_instance;
  }
  var TexTalkTokenType$LSquare_instance;
  function TexTalkTokenType$LSquare_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$LSquare_instance;
  }
  var TexTalkTokenType$RSquare_instance;
  function TexTalkTokenType$RSquare_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$RSquare_instance;
  }
  var TexTalkTokenType$LCurly_instance;
  function TexTalkTokenType$LCurly_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$LCurly_instance;
  }
  var TexTalkTokenType$RCurly_instance;
  function TexTalkTokenType$RCurly_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$RCurly_instance;
  }
  var TexTalkTokenType$Operator_instance;
  function TexTalkTokenType$Operator_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$Operator_instance;
  }
  var TexTalkTokenType$Identifier_instance;
  function TexTalkTokenType$Identifier_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$Identifier_instance;
  }
  var TexTalkTokenType$Comma_instance;
  function TexTalkTokenType$Comma_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$Comma_instance;
  }
  var TexTalkTokenType$Period_instance;
  function TexTalkTokenType$Period_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$Period_instance;
  }
  var TexTalkTokenType$Colon_instance;
  function TexTalkTokenType$Colon_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$Colon_instance;
  }
  var TexTalkTokenType$Underscore_instance;
  function TexTalkTokenType$Underscore_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$Underscore_instance;
  }
  var TexTalkTokenType$Caret_instance;
  function TexTalkTokenType$Caret_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$Caret_instance;
  }
  var TexTalkTokenType$ColonEquals_instance;
  function TexTalkTokenType$ColonEquals_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$ColonEquals_instance;
  }
  var TexTalkTokenType$Is_instance;
  function TexTalkTokenType$Is_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$Is_instance;
  }
  var TexTalkTokenType$Invalid_instance;
  function TexTalkTokenType$Invalid_getInstance() {
    TexTalkTokenType_initFields();
    return TexTalkTokenType$Invalid_instance;
  }
  TexTalkTokenType.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'TexTalkTokenType',
    interfaces: [Enum]
  };
  function TexTalkTokenType$values() {
    return [TexTalkTokenType$Backslash_getInstance(), TexTalkTokenType$LParen_getInstance(), TexTalkTokenType$RParen_getInstance(), TexTalkTokenType$LSquare_getInstance(), TexTalkTokenType$RSquare_getInstance(), TexTalkTokenType$LCurly_getInstance(), TexTalkTokenType$RCurly_getInstance(), TexTalkTokenType$Operator_getInstance(), TexTalkTokenType$Identifier_getInstance(), TexTalkTokenType$Comma_getInstance(), TexTalkTokenType$Period_getInstance(), TexTalkTokenType$Colon_getInstance(), TexTalkTokenType$Underscore_getInstance(), TexTalkTokenType$Caret_getInstance(), TexTalkTokenType$ColonEquals_getInstance(), TexTalkTokenType$Is_getInstance(), TexTalkTokenType$Invalid_getInstance()];
  }
  TexTalkTokenType.values = TexTalkTokenType$values;
  function TexTalkTokenType$valueOf(name) {
    switch (name) {
      case 'Backslash':
        return TexTalkTokenType$Backslash_getInstance();
      case 'LParen':
        return TexTalkTokenType$LParen_getInstance();
      case 'RParen':
        return TexTalkTokenType$RParen_getInstance();
      case 'LSquare':
        return TexTalkTokenType$LSquare_getInstance();
      case 'RSquare':
        return TexTalkTokenType$RSquare_getInstance();
      case 'LCurly':
        return TexTalkTokenType$LCurly_getInstance();
      case 'RCurly':
        return TexTalkTokenType$RCurly_getInstance();
      case 'Operator':
        return TexTalkTokenType$Operator_getInstance();
      case 'Identifier':
        return TexTalkTokenType$Identifier_getInstance();
      case 'Comma':
        return TexTalkTokenType$Comma_getInstance();
      case 'Period':
        return TexTalkTokenType$Period_getInstance();
      case 'Colon':
        return TexTalkTokenType$Colon_getInstance();
      case 'Underscore':
        return TexTalkTokenType$Underscore_getInstance();
      case 'Caret':
        return TexTalkTokenType$Caret_getInstance();
      case 'ColonEquals':
        return TexTalkTokenType$ColonEquals_getInstance();
      case 'Is':
        return TexTalkTokenType$Is_getInstance();
      case 'Invalid':
        return TexTalkTokenType$Invalid_getInstance();
      default:throwISE('No enum constant mathlingua.common.textalk.TexTalkTokenType.' + name);
    }
  }
  TexTalkTokenType.valueOf_61zpoe$ = TexTalkTokenType$valueOf;
  function TexTalkLexer() {
  }
  TexTalkLexer.$metadata$ = {
    kind: Kind_INTERFACE,
    simpleName: 'TexTalkLexer',
    interfaces: []
  };
  function newTexTalkLexer(text) {
    return new TexTalkLexerImpl(text);
  }
  function TexTalkLexerImpl(text) {
    this.errors_rts390$_0 = null;
    this.tokens_0 = null;
    this.index_0 = 0;
    var tmp$, tmp$_0, tmp$_1;
    this.errors_rts390$_0 = ArrayList_init();
    this.tokens_0 = ArrayList_init();
    this.index_0 = 0;
    var i = 0;
    var line = 0;
    var column = -1;
    while (i < text.length) {
      var c = text.charCodeAt((tmp$ = i, i = tmp$ + 1 | 0, tmp$));
      column = column + 1 | 0;
      if (c === 10) {
        line = line + 1 | 0;
        column = 0;
      }
       else if (c === 92) {
        this.tokens_0.add_11rb$(new TexTalkToken('' + String.fromCharCode(toBoxedChar(c)), TexTalkTokenType$Backslash_getInstance(), line, column));
      }
       else if (c === 105 && i < text.length && text.charCodeAt(i) === 115) {
        this.tokens_0.add_11rb$(new TexTalkToken('is', TexTalkTokenType$Is_getInstance(), line, column));
        i = i + 1 | 0;
        column = column + 1 | 0;
      }
       else if (c === 58 && i < text.length && text.charCodeAt(i) === 61) {
        this.tokens_0.add_11rb$(new TexTalkToken(':=', TexTalkTokenType$ColonEquals_getInstance(), line, column));
        i = i + 1 | 0;
        column = column + 1 | 0;
      }
       else if (c === 58) {
        this.tokens_0.add_11rb$(new TexTalkToken('' + String.fromCharCode(toBoxedChar(c)), TexTalkTokenType$Colon_getInstance(), line, column));
      }
       else if (c === 46) {
        this.tokens_0.add_11rb$(new TexTalkToken('' + String.fromCharCode(toBoxedChar(c)), TexTalkTokenType$Period_getInstance(), line, column));
      }
       else if (c === 40) {
        this.tokens_0.add_11rb$(new TexTalkToken('' + String.fromCharCode(toBoxedChar(c)), TexTalkTokenType$LParen_getInstance(), line, column));
      }
       else if (c === 41) {
        this.tokens_0.add_11rb$(new TexTalkToken('' + String.fromCharCode(toBoxedChar(c)), TexTalkTokenType$RParen_getInstance(), line, column));
      }
       else if (c === 91) {
        this.tokens_0.add_11rb$(new TexTalkToken('' + String.fromCharCode(toBoxedChar(c)), TexTalkTokenType$LSquare_getInstance(), line, column));
      }
       else if (c === 93) {
        this.tokens_0.add_11rb$(new TexTalkToken('' + String.fromCharCode(toBoxedChar(c)), TexTalkTokenType$RSquare_getInstance(), line, column));
      }
       else if (c === 123) {
        this.tokens_0.add_11rb$(new TexTalkToken('' + String.fromCharCode(toBoxedChar(c)), TexTalkTokenType$LCurly_getInstance(), line, column));
      }
       else if (c === 125) {
        this.tokens_0.add_11rb$(new TexTalkToken('' + String.fromCharCode(toBoxedChar(c)), TexTalkTokenType$RCurly_getInstance(), line, column));
      }
       else if (c === 95) {
        this.tokens_0.add_11rb$(new TexTalkToken('' + String.fromCharCode(toBoxedChar(c)), TexTalkTokenType$Underscore_getInstance(), line, column));
      }
       else if (c === 94) {
        this.tokens_0.add_11rb$(new TexTalkToken('' + String.fromCharCode(toBoxedChar(c)), TexTalkTokenType$Caret_getInstance(), line, column));
      }
       else if (c === 44) {
        this.tokens_0.add_11rb$(new TexTalkToken('' + String.fromCharCode(toBoxedChar(c)), TexTalkTokenType$Comma_getInstance(), line, column));
      }
       else if (this.isLetterOrDigit_0(c)) {
        var id = new StringBuilder('' + String.fromCharCode(toBoxedChar(c)));
        while (i < text.length && this.isLetterOrDigit_0(text.charCodeAt(i))) {
          id.append_s8itvh$(text.charCodeAt((tmp$_0 = i, i = tmp$_0 + 1 | 0, tmp$_0)));
          column = column + 1 | 0;
        }
        this.tokens_0.add_11rb$(new TexTalkToken(id.toString(), TexTalkTokenType$Identifier_getInstance(), line, column));
      }
       else if (this.isOpChar_0(c)) {
        var op = new StringBuilder('' + String.fromCharCode(toBoxedChar(c)));
        while (i < text.length && this.isOpChar_0(text.charCodeAt(i))) {
          op.append_s8itvh$(text.charCodeAt((tmp$_1 = i, i = tmp$_1 + 1 | 0, tmp$_1)));
          column = column + 1 | 0;
        }
        this.tokens_0.add_11rb$(new TexTalkToken(op.toString(), TexTalkTokenType$Operator_getInstance(), line, column));
      }
       else if (c !== 32) {
        this.errors.add_11rb$(new ParseError('Unrecognized character ' + String.fromCharCode(c), line, column));
      }
    }
  }
  Object.defineProperty(TexTalkLexerImpl.prototype, 'errors', {
    get: function () {
      return this.errors_rts390$_0;
    }
  });
  TexTalkLexerImpl.prototype.hasNext = function () {
    return this.index_0 < this.tokens_0.size;
  };
  TexTalkLexerImpl.prototype.hasNextNext = function () {
    return (this.index_0 + 1 | 0) < this.tokens_0.size;
  };
  TexTalkLexerImpl.prototype.peek = function () {
    return this.tokens_0.get_za3lpa$(this.index_0);
  };
  TexTalkLexerImpl.prototype.peekPeek = function () {
    return this.tokens_0.get_za3lpa$(this.index_0 + 1 | 0);
  };
  TexTalkLexerImpl.prototype.next = function () {
    var result = this.peek();
    this.index_0 = this.index_0 + 1 | 0;
    return result;
  };
  TexTalkLexerImpl.prototype.isOpChar_0 = function (c) {
    return c === 33 || c === 64 || c === 37 || c === 38 || c === 42 || c === 45 || c === 43 || c === 61 || c === 124 || c === 47 || c === 60 || c === 62;
  };
  TexTalkLexerImpl.prototype.isLetterOrDigit_0 = function (c) {
    return Regex_init('[a-zA-Z0-9]+').matches_6bul2c$(String.fromCharCode(c));
  };
  TexTalkLexerImpl.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'TexTalkLexerImpl',
    interfaces: [TexTalkLexer]
  };
  function TexTalkParser() {
  }
  TexTalkParser.$metadata$ = {
    kind: Kind_INTERFACE,
    simpleName: 'TexTalkParser',
    interfaces: []
  };
  function TexTalkParseResult(root, errors) {
    this.root = root;
    this.errors = errors;
  }
  TexTalkParseResult.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'TexTalkParseResult',
    interfaces: []
  };
  TexTalkParseResult.prototype.component1 = function () {
    return this.root;
  };
  TexTalkParseResult.prototype.component2 = function () {
    return this.errors;
  };
  TexTalkParseResult.prototype.copy_cyo9yp$ = function (root, errors) {
    return new TexTalkParseResult(root === void 0 ? this.root : root, errors === void 0 ? this.errors : errors);
  };
  TexTalkParseResult.prototype.toString = function () {
    return 'TexTalkParseResult(root=' + Kotlin.toString(this.root) + (', errors=' + Kotlin.toString(this.errors)) + ')';
  };
  TexTalkParseResult.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.root) | 0;
    result = result * 31 + Kotlin.hashCode(this.errors) | 0;
    return result;
  };
  TexTalkParseResult.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.root, other.root) && Kotlin.equals(this.errors, other.errors)))));
  };
  function newTexTalkParser() {
    return new TexTalkParserImpl();
  }
  function TexTalkParserImpl() {
  }
  TexTalkParserImpl.prototype.parse_2mg13h$ = function (texTalkLexer) {
    var worker = new TexTalkParserImpl$ParserWorker(texTalkLexer);
    var root = worker.parse();
    var errors = worker.getErrors();
    return new TexTalkParseResult(root, errors);
  };
  function TexTalkParserImpl$ParserWorker(texTalkLexer) {
    this.texTalkLexer_0 = texTalkLexer;
    this.errors_0 = null;
    this.errors_0 = ArrayList_init();
  }
  TexTalkParserImpl$ParserWorker.prototype.getErrors = function () {
    return this.errors_0;
  };
  TexTalkParserImpl$ParserWorker.prototype.parse = function () {
    var tmp$, tmp$_0;
    var exp = (tmp$ = this.expression_0(null)) != null ? tmp$ : new ExpressionNode(emptyList());
    return Kotlin.isType(tmp$_0 = this.resolveColonEqualsNode_0(this.resolveIsNode_0(exp)), ExpressionNode) ? tmp$_0 : throwCCE();
  };
  TexTalkParserImpl$ParserWorker.prototype.resolveIsNode_0 = function (node) {
    var tmp$;
    if (!Kotlin.isType(node, ExpressionNode)) {
      return node;
    }
    var isIndex = -1;
    tmp$ = node.children.size;
    for (var i = 0; i < tmp$; i++) {
      var child = node.children.get_za3lpa$(i);
      if (Kotlin.isType(child, TextNode) && child.type === NodeType$Is_getInstance()) {
        if (isIndex < 0) {
          isIndex = i;
        }
         else {
          this.errors_0.add_11rb$(new ParseError("A statement can only contain one 'is' statement", -1, -1));
        }
      }
    }
    if (isIndex < 0) {
      return node;
    }
    var lhs = this.parameters_0(node.children, 0, isIndex);
    var rhs = this.parameters_0(node.children, isIndex + 1 | 0, node.children.size);
    return new ExpressionNode(listOf(new IsNode(lhs, rhs)));
  };
  TexTalkParserImpl$ParserWorker.prototype.resolveColonEqualsNode_0 = function (node) {
    var tmp$;
    if (!Kotlin.isType(node, ExpressionNode)) {
      return node;
    }
    var colonEqualsIndex = -1;
    tmp$ = node.children.size;
    for (var i = 0; i < tmp$; i++) {
      var child = node.children.get_za3lpa$(i);
      if (Kotlin.isType(child, TextNode) && child.type === NodeType$ColonEquals_getInstance()) {
        if (colonEqualsIndex < 0) {
          colonEqualsIndex = i;
        }
         else {
          this.errors_0.add_11rb$(new ParseError("A statement can only contain one ':='", -1, -1));
        }
      }
    }
    if (colonEqualsIndex < 0) {
      return node;
    }
    var lhs = this.parameters_0(node.children, 0, colonEqualsIndex);
    var rhs = this.parameters_0(node.children, colonEqualsIndex + 1 | 0, node.children.size);
    return new ExpressionNode(listOf(new ColonEqualsNode(lhs, rhs)));
  };
  TexTalkParserImpl$ParserWorker.prototype.parameters_0 = function (nodes, startInc, endEx) {
    var tmp$;
    var parts = ArrayList_init();
    var i = startInc;
    while (i < endEx) {
      var items = ArrayList_init();
      while (i < endEx && nodes.get_za3lpa$(i).type !== NodeType$Comma_getInstance()) {
        items.add_11rb$(this.resolveIsNode_0(nodes.get_za3lpa$((tmp$ = i, i = tmp$ + 1 | 0, tmp$))));
      }
      if (i < endEx && nodes.get_za3lpa$(i).type !== NodeType$Comma_getInstance()) {
        throw RuntimeException_init('Expected a Comma but found ' + nodes.get_za3lpa$(i).type);
      }
      i = i + 1 | 0;
      parts.add_11rb$(new ExpressionNode(items));
    }
    return new ParametersNode(parts);
  };
  TexTalkParserImpl$ParserWorker.prototype.command_0 = function () {
    if (!this.has_0(TexTalkTokenType$Backslash_getInstance())) {
      return null;
    }
    var backSlash = this.expect_0(TexTalkTokenType$Backslash_getInstance());
    var parts = ArrayList_init();
    while (this.texTalkLexer_0.hasNext()) {
      var part = this.commandPart_0();
      if (part == null) {
        this.errors_0.add_11rb$(new ParseError('Missing a command part', backSlash.row, backSlash.column));
      }
       else {
        parts.add_11rb$(part);
      }
      if (this.has_0(TexTalkTokenType$Period_getInstance())) {
        this.expect_0(TexTalkTokenType$Period_getInstance());
      }
       else {
        break;
      }
    }
    if (parts.isEmpty()) {
      this.errors_0.add_11rb$(new ParseError('Expected at least one command part', backSlash.row, backSlash.column));
    }
    return new Command(parts);
  };
  TexTalkParserImpl$ParserWorker.prototype.commandPart_0 = function () {
    if (!this.has_0(TexTalkTokenType$Identifier_getInstance())) {
      return null;
    }
    var name = this.text_0(TexTalkTokenType$Identifier_getInstance(), NodeType$Identifier_getInstance());
    var square = this.group_0(NodeType$SquareGroup_getInstance());
    var subSup = this.subSup_0();
    var groups = ArrayList_init();
    var startGroup = null;
    var paren = this.group_0(NodeType$ParenGroup_getInstance());
    if (paren != null) {
      startGroup = paren;
    }
    if (startGroup == null) {
      var curly = this.group_0(NodeType$CurlyGroup_getInstance());
      if (curly != null) {
        startGroup = curly;
      }
    }
    if (startGroup != null) {
      groups.add_11rb$(startGroup);
      while (this.texTalkLexer_0.hasNext()) {
        var grp = this.group_0(startGroup.type);
        if (grp == null) {
          break;
        }
        groups.add_11rb$(grp);
      }
    }
    var namedGroups = ArrayList_init();
    if (this.has_0(TexTalkTokenType$Colon_getInstance())) {
      this.expect_0(TexTalkTokenType$Colon_getInstance());
      while (this.texTalkLexer_0.hasNext()) {
        var namedGrp = this.namedGroup_0();
        if (namedGrp == null) {
          break;
        }
        namedGroups.add_11rb$(namedGrp);
      }
    }
    return new CommandPart(ensureNotNull(name), square, subSup, groups, namedGroups);
  };
  TexTalkParserImpl$ParserWorker.prototype.subSup_0 = function () {
    var tmp$;
    var sub = this.sub_0();
    var sup = this.sup_0();
    if (sub == null && sup == null) {
      tmp$ = null;
    }
     else
      tmp$ = new SubSupNode(sub, sup);
    return tmp$;
  };
  TexTalkParserImpl$ParserWorker.prototype.sub_0 = function () {
    if (!this.has_0(TexTalkTokenType$Underscore_getInstance())) {
      return null;
    }
    var tmp$ = this.expect_0(TexTalkTokenType$Underscore_getInstance());
    var row = tmp$.component3()
    , column = tmp$.component4();
    var grp = null;
    var curly = this.group_0(NodeType$CurlyGroup_getInstance());
    if (curly != null) {
      grp = curly;
    }
    if (grp == null) {
      var paren = this.group_0(NodeType$ParenGroup_getInstance());
      if (paren != null) {
        grp = paren;
      }
    }
    if (grp == null) {
      this.errors_0.add_11rb$(new ParseError('Expected a value with an underscore', row, column));
      grp = new GroupNode(NodeType$CurlyGroup_getInstance(), new ParametersNode(emptyList()));
    }
    return grp;
  };
  TexTalkParserImpl$ParserWorker.prototype.sup_0 = function () {
    if (!this.has_0(TexTalkTokenType$Caret_getInstance())) {
      return null;
    }
    var tmp$ = this.expect_0(TexTalkTokenType$Caret_getInstance());
    var row = tmp$.component3()
    , column = tmp$.component4();
    var grp = null;
    var curly = this.group_0(NodeType$CurlyGroup_getInstance());
    if (curly != null) {
      grp = curly;
    }
    if (grp == null) {
      var paren = this.group_0(NodeType$ParenGroup_getInstance());
      if (paren != null) {
        grp = paren;
      }
    }
    if (grp == null) {
      this.errors_0.add_11rb$(new ParseError('Expected a value with a caret', row, column));
      grp = new GroupNode(NodeType$CurlyGroup_getInstance(), new ParametersNode(emptyList()));
    }
    return grp;
  };
  TexTalkParserImpl$ParserWorker.prototype.group_0 = function (nodeType) {
    var startType;
    var endType;
    switch (nodeType.name) {
      case 'ParenGroup':
        startType = TexTalkTokenType$LParen_getInstance();
        endType = TexTalkTokenType$RParen_getInstance();
        break;
      case 'SquareGroup':
        startType = TexTalkTokenType$LSquare_getInstance();
        endType = TexTalkTokenType$RSquare_getInstance();
        break;
      case 'CurlyGroup':
        startType = TexTalkTokenType$LCurly_getInstance();
        endType = TexTalkTokenType$RCurly_getInstance();
        break;
      default:throw RuntimeException_init('Unrecognized group type ' + nodeType);
    }
    if (!this.has_0(startType)) {
      return null;
    }
    var expressions = ArrayList_init();
    this.expect_0(startType);
    var terminators = HashSet_init();
    terminators.add_11rb$(TexTalkTokenType$Comma_getInstance());
    terminators.add_11rb$(endType);
    var firstExp = this.expression_0(terminators);
    if (firstExp != null) {
      expressions.add_11rb$(firstExp);
    }
    while (this.has_0(TexTalkTokenType$Comma_getInstance())) {
      this.texTalkLexer_0.next();
      var exp = this.expression_0(terminators);
      if (exp == null)
        break;
      expressions.add_11rb$(exp);
    }
    this.expect_0(endType);
    return new GroupNode(nodeType, new ParametersNode(expressions));
  };
  TexTalkParserImpl$ParserWorker.prototype.namedGroup_0 = function () {
    var isNamedGroup = this.texTalkLexer_0.hasNext() && this.texTalkLexer_0.hasNextNext() && this.texTalkLexer_0.peek().tokenType === TexTalkTokenType$Identifier_getInstance() && this.texTalkLexer_0.peekPeek().tokenType === TexTalkTokenType$LCurly_getInstance();
    if (!isNamedGroup) {
      return null;
    }
    var rawText = this.text_0(TexTalkTokenType$Identifier_getInstance(), NodeType$Identifier_getInstance());
    var text;
    if (rawText != null) {
      text = rawText;
    }
     else {
      this.errors_0.add_11rb$(new ParseError('Expected an identifier in a named group', -1, -1));
      text = new TextNode(NodeType$Identifier_getInstance(), 'INVALID');
    }
    var rawGroup = this.group_0(NodeType$CurlyGroup_getInstance());
    var group;
    if (rawGroup != null) {
      group = rawGroup;
    }
     else {
      this.errors_0.add_11rb$(new ParseError('Expected a group in a named group', -1, -1));
      group = new GroupNode(NodeType$CurlyGroup_getInstance(), new ParametersNode(emptyList()));
    }
    return new NamedGroupNode(text, group);
  };
  TexTalkParserImpl$ParserWorker.prototype.text_0 = function (tokenType, nodeType) {
    var tmp$;
    if (!this.has_0(tokenType)) {
      tmp$ = null;
    }
     else
      tmp$ = new TextNode(nodeType, this.texTalkLexer_0.next().text);
    return tmp$;
  };
  TexTalkParserImpl$ParserWorker.prototype.expression_0 = function (terminators) {
    var tmp$, tmp$_0, tmp$_1, tmp$_2, tmp$_3, tmp$_4, tmp$_5, tmp$_6, tmp$_7, tmp$_8;
    var nodes = ArrayList_init();
    while (this.texTalkLexer_0.hasNext() && (terminators == null || !terminators.contains_11rb$(this.texTalkLexer_0.peek().tokenType))) {
      var child = (tmp$_7 = (tmp$_6 = (tmp$_5 = (tmp$_4 = (tmp$_3 = (tmp$_2 = (tmp$_1 = (tmp$_0 = (tmp$ = this.command_0()) != null ? tmp$ : this.group_0(NodeType$ParenGroup_getInstance())) != null ? tmp$_0 : this.group_0(NodeType$CurlyGroup_getInstance())) != null ? tmp$_1 : this.text_0(TexTalkTokenType$Is_getInstance(), NodeType$Is_getInstance())) != null ? tmp$_2 : this.text_0(TexTalkTokenType$Identifier_getInstance(), NodeType$Identifier_getInstance())) != null ? tmp$_3 : this.text_0(TexTalkTokenType$Operator_getInstance(), NodeType$Operator_getInstance())) != null ? tmp$_4 : this.text_0(TexTalkTokenType$Comma_getInstance(), NodeType$Comma_getInstance())) != null ? tmp$_5 : this.text_0(TexTalkTokenType$Caret_getInstance(), NodeType$Operator_getInstance())) != null ? tmp$_6 : this.text_0(TexTalkTokenType$Underscore_getInstance(), NodeType$Operator_getInstance())) != null ? tmp$_7 : this.text_0(TexTalkTokenType$ColonEquals_getInstance(), NodeType$ColonEquals_getInstance());
      if (child == null) {
        var peek = this.texTalkLexer_0.peek();
        this.errors_0.add_11rb$(new ParseError('Unexpected token ' + peek.text, peek.row, peek.column));
        this.texTalkLexer_0.next();
      }
       else {
        nodes.add_11rb$(child);
      }
    }
    if (nodes.isEmpty()) {
      tmp$_8 = null;
    }
     else
      tmp$_8 = new ExpressionNode(nodes);
    return tmp$_8;
  };
  TexTalkParserImpl$ParserWorker.prototype.expect_0 = function (tokenType) {
    var tmp$;
    if (this.has_0(tokenType)) {
      return this.texTalkLexer_0.next();
    }
     else {
      if (this.texTalkLexer_0.hasNext()) {
        tmp$ = "Expected a token of type '" + toString(tokenType) + "' but " + "found type '" + toString(this.texTalkLexer_0.peek().type) + "' for text '" + this.texTalkLexer_0.peek().text + "' (Line: " + toString(this.texTalkLexer_0.peek().row + 1 | 0) + ', Column: ' + toString(this.texTalkLexer_0.peek().column + 1 | 0) + ')';
      }
       else {
        tmp$ = 'Expected a token of type ' + tokenType + ' but found the end of input';
      }
      var message = tmp$;
      this.errors_0.add_11rb$(new ParseError(message, -1, -1));
      return new TexTalkToken('INVALID', TexTalkTokenType$Invalid_getInstance(), -1, -1);
    }
  };
  TexTalkParserImpl$ParserWorker.prototype.has_0 = function (tokenType) {
    return this.texTalkLexer_0.hasNext() && this.texTalkLexer_0.peek().tokenType === tokenType;
  };
  TexTalkParserImpl$ParserWorker.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ParserWorker',
    interfaces: []
  };
  TexTalkParserImpl.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'TexTalkParserImpl',
    interfaces: [TexTalkParser]
  };
  var package$mathlingua = _.mathlingua || (_.mathlingua = {});
  var package$common = package$mathlingua.common || (package$mathlingua.common = {});
  package$common.MathLinguaResult = MathLinguaResult;
  package$common.MathLingua = MathLingua;
  package$common.findAllSignaturesImpl_paasgs$ = findAllSignaturesImpl;
  package$common.ParseError = ParseError;
  Object.defineProperty(Validation, 'Companion', {
    get: Validation$Companion_getInstance
  });
  package$common.Validation = Validation;
  var package$chalktalk = package$common.chalktalk || (package$common.chalktalk = {});
  var package$phase1 = package$chalktalk.phase1 || (package$chalktalk.phase1 = {});
  package$phase1.ChalkTalkLexer = ChalkTalkLexer;
  package$phase1.newChalkTalkLexer_61zpoe$ = newChalkTalkLexer;
  package$phase1.ChalkTalkParser = ChalkTalkParser;
  package$phase1.ChalkTalkParseResult = ChalkTalkParseResult;
  package$phase1.newChalkTalkParser = newChalkTalkParser;
  var package$ast = package$phase1.ast || (package$phase1.ast = {});
  package$ast.ChalkTalkNode = ChalkTalkNode;
  package$ast.Root = Root;
  package$ast.Argument = Argument;
  package$ast.Section = Section;
  Object.defineProperty(package$ast, 'AstUtils', {
    get: AstUtils_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'DotSpace', {
    get: ChalkTalkTokenType$DotSpace_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'Name', {
    get: ChalkTalkTokenType$Name_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'Colon', {
    get: ChalkTalkTokenType$Colon_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'String', {
    get: ChalkTalkTokenType$String_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'Statement', {
    get: ChalkTalkTokenType$Statement_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'Id', {
    get: ChalkTalkTokenType$Id_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'Comma', {
    get: ChalkTalkTokenType$Comma_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'Begin', {
    get: ChalkTalkTokenType$Begin_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'End', {
    get: ChalkTalkTokenType$End_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'Linebreak', {
    get: ChalkTalkTokenType$Linebreak_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'Invalid', {
    get: ChalkTalkTokenType$Invalid_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'Equals', {
    get: ChalkTalkTokenType$Equals_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'ColonEquals', {
    get: ChalkTalkTokenType$ColonEquals_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'LParen', {
    get: ChalkTalkTokenType$LParen_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'RParen', {
    get: ChalkTalkTokenType$RParen_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'LCurly', {
    get: ChalkTalkTokenType$LCurly_getInstance
  });
  Object.defineProperty(ChalkTalkTokenType, 'RCurly', {
    get: ChalkTalkTokenType$RCurly_getInstance
  });
  package$ast.ChalkTalkTokenType = ChalkTalkTokenType;
  package$ast.ChalkTalkTarget = ChalkTalkTarget;
  package$ast.TupleItem = TupleItem;
  package$ast.AssignmentRhs = AssignmentRhs;
  package$ast.ChalkTalkToken = ChalkTalkToken;
  package$ast.Mapping = Mapping;
  package$ast.Group = Group;
  package$ast.Assignment = Assignment;
  package$ast.Tuple = Tuple;
  package$ast.Abstraction = Abstraction;
  package$ast.Aggregate = Aggregate;
  Object.defineProperty(AliasSection, 'Companion', {
    get: AliasSection$Companion_getInstance
  });
  var package$phase2 = package$chalktalk.phase2 || (package$chalktalk.phase2 = {});
  package$phase2.AliasSection = AliasSection;
  package$phase2.indentedString_qta3xh$ = indentedString;
  Object.defineProperty(Clause, 'Companion', {
    get: Clause$Companion_getInstance
  });
  package$phase2.Clause = Clause;
  package$phase2.Target = Target;
  Object.defineProperty(AbstractionNode, 'Companion', {
    get: AbstractionNode$Companion_getInstance
  });
  package$phase2.AbstractionNode = AbstractionNode;
  Object.defineProperty(AggregateNode, 'Companion', {
    get: AggregateNode$Companion_getInstance
  });
  package$phase2.AggregateNode = AggregateNode;
  Object.defineProperty(TupleNode, 'Companion', {
    get: TupleNode$Companion_getInstance
  });
  package$phase2.TupleNode = TupleNode;
  Object.defineProperty(AssignmentNode, 'Companion', {
    get: AssignmentNode$Companion_getInstance
  });
  package$phase2.AssignmentNode = AssignmentNode;
  Object.defineProperty(MappingNode, 'Companion', {
    get: MappingNode$Companion_getInstance
  });
  package$phase2.MappingNode = MappingNode;
  Object.defineProperty(Identifier, 'Companion', {
    get: Identifier$Companion_getInstance
  });
  package$phase2.Identifier = Identifier;
  Object.defineProperty(Statement, 'Companion', {
    get: Statement$Companion_getInstance
  });
  package$phase2.Statement = Statement;
  Object.defineProperty(Text, 'Companion', {
    get: Text$Companion_getInstance
  });
  package$phase2.Text = Text;
  Object.defineProperty(ExistsGroup, 'Companion', {
    get: ExistsGroup$Companion_getInstance
  });
  package$phase2.ExistsGroup = ExistsGroup;
  Object.defineProperty(IfGroup, 'Companion', {
    get: IfGroup$Companion_getInstance
  });
  package$phase2.IfGroup = IfGroup;
  Object.defineProperty(IffGroup, 'Companion', {
    get: IffGroup$Companion_getInstance
  });
  package$phase2.IffGroup = IffGroup;
  Object.defineProperty(ForGroup, 'Companion', {
    get: ForGroup$Companion_getInstance
  });
  package$phase2.ForGroup = ForGroup;
  Object.defineProperty(NotGroup, 'Companion', {
    get: NotGroup$Companion_getInstance
  });
  package$phase2.NotGroup = NotGroup;
  Object.defineProperty(OrGroup, 'Companion', {
    get: OrGroup$Companion_getInstance
  });
  package$phase2.OrGroup = OrGroup;
  package$phase2.firstSectionMatchesName_x3cv99$ = firstSectionMatchesName;
  package$phase2.validateSingleSectionGroup_rkyvc5$ = validateSingleSectionGroup;
  package$phase2.toCode_o9yfac$ = toCode;
  package$phase2.toCode_tsx3j7$ = toCode_0;
  package$phase2.ClauseListSection = ClauseListSection;
  Object.defineProperty(package$phase2, 'ClauseListValidator', {
    get: ClauseListValidator_getInstance
  });
  package$phase2.Phase2Node = Phase2Node;
  Object.defineProperty(Document, 'Companion', {
    get: Document$Companion_getInstance
  });
  package$phase2.Document = Document;
  Object.defineProperty(DefinesGroup, 'Companion', {
    get: DefinesGroup$Companion_getInstance
  });
  package$phase2.DefinesGroup = DefinesGroup;
  Object.defineProperty(RepresentsGroup, 'Companion', {
    get: RepresentsGroup$Companion_getInstance
  });
  package$phase2.RepresentsGroup = RepresentsGroup;
  Object.defineProperty(ResultGroup, 'Companion', {
    get: ResultGroup$Companion_getInstance
  });
  package$phase2.ResultGroup = ResultGroup;
  Object.defineProperty(AxiomGroup, 'Companion', {
    get: AxiomGroup$Companion_getInstance
  });
  package$phase2.AxiomGroup = AxiomGroup;
  Object.defineProperty(ConjectureGroup, 'Companion', {
    get: ConjectureGroup$Companion_getInstance
  });
  package$phase2.ConjectureGroup = ConjectureGroup;
  package$phase2.toCode_2d2cwo$ = toCode_1;
  package$phase2.validateResultLikeGroup_g6v280$ = validateResultLikeGroup;
  package$phase2.validateDefinesLikeGroup_yceizu$ = validateDefinesLikeGroup;
  package$phase2.getOrNull_t9ocha$ = getOrNull;
  package$phase2.getSignature_c9m4n2$ = getSignature;
  package$phase2.findAllStatementSignatures_c9m4n2$ = findAllStatementSignatures;
  package$phase2.getMergedCommandSignature_pafvx0$ = getMergedCommandSignature;
  package$phase2.getCommandSignature_mwzhn3$ = getCommandSignature;
  package$phase2.callOrNull_h43q6c$ = callOrNull;
  Object.defineProperty(MetaDataSection, 'Companion', {
    get: MetaDataSection$Companion_getInstance
  });
  package$phase2.MetaDataSection = MetaDataSection;
  package$phase2.appendClauseArgs_ddrmb7$ = appendClauseArgs;
  package$phase2.appendTargetArgs_wz7ljp$ = appendTargetArgs;
  Object.defineProperty(AssumingSection, 'Companion', {
    get: AssumingSection$Companion_getInstance
  });
  package$phase2.AssumingSection = AssumingSection;
  Object.defineProperty(DefinesSection, 'Companion', {
    get: DefinesSection$Companion_getInstance
  });
  package$phase2.DefinesSection = DefinesSection;
  Object.defineProperty(RefinesSection, 'Companion', {
    get: RefinesSection$Companion_getInstance
  });
  package$phase2.RefinesSection = RefinesSection;
  Object.defineProperty(RepresentsSection, 'Companion', {
    get: RepresentsSection$Companion_getInstance
  });
  package$phase2.RepresentsSection = RepresentsSection;
  Object.defineProperty(ExistsSection, 'Companion', {
    get: ExistsSection$Companion_getInstance
  });
  package$phase2.ExistsSection = ExistsSection;
  Object.defineProperty(ForSection, 'Companion', {
    get: ForSection$Companion_getInstance
  });
  package$phase2.ForSection = ForSection;
  Object.defineProperty(MeansSection, 'Companion', {
    get: MeansSection$Companion_getInstance
  });
  package$phase2.MeansSection = MeansSection;
  Object.defineProperty(ResultSection, 'Companion', {
    get: ResultSection$Companion_getInstance
  });
  package$phase2.ResultSection = ResultSection;
  Object.defineProperty(AxiomSection, 'Companion', {
    get: AxiomSection$Companion_getInstance
  });
  package$phase2.AxiomSection = AxiomSection;
  Object.defineProperty(ConjectureSection, 'Companion', {
    get: ConjectureSection$Companion_getInstance
  });
  package$phase2.ConjectureSection = ConjectureSection;
  Object.defineProperty(SuchThatSection, 'Companion', {
    get: SuchThatSection$Companion_getInstance
  });
  package$phase2.SuchThatSection = SuchThatSection;
  Object.defineProperty(ThatSection, 'Companion', {
    get: ThatSection$Companion_getInstance
  });
  package$phase2.ThatSection = ThatSection;
  Object.defineProperty(IfSection, 'Companion', {
    get: IfSection$Companion_getInstance
  });
  package$phase2.IfSection = IfSection;
  Object.defineProperty(IffSection, 'Companion', {
    get: IffSection$Companion_getInstance
  });
  package$phase2.IffSection = IffSection;
  Object.defineProperty(ThenSection, 'Companion', {
    get: ThenSection$Companion_getInstance
  });
  package$phase2.ThenSection = ThenSection;
  Object.defineProperty(WhereSection, 'Companion', {
    get: WhereSection$Companion_getInstance
  });
  package$phase2.WhereSection = WhereSection;
  Object.defineProperty(NotSection, 'Companion', {
    get: NotSection$Companion_getInstance
  });
  package$phase2.NotSection = NotSection;
  Object.defineProperty(OrSection, 'Companion', {
    get: OrSection$Companion_getInstance
  });
  package$phase2.OrSection = OrSection;
  Object.defineProperty(package$phase2, 'SectionIdentifier', {
    get: SectionIdentifier_getInstance
  });
  package$phase2.TargetListSection = TargetListSection;
  Object.defineProperty(package$phase2, 'TargetListValidator', {
    get: TargetListValidator_getInstance
  });
  Object.defineProperty(NodeType, 'Token', {
    get: NodeType$Token_getInstance
  });
  Object.defineProperty(NodeType, 'Identifier', {
    get: NodeType$Identifier_getInstance
  });
  Object.defineProperty(NodeType, 'Operator', {
    get: NodeType$Operator_getInstance
  });
  Object.defineProperty(NodeType, 'ParenGroup', {
    get: NodeType$ParenGroup_getInstance
  });
  Object.defineProperty(NodeType, 'SquareGroup', {
    get: NodeType$SquareGroup_getInstance
  });
  Object.defineProperty(NodeType, 'CurlyGroup', {
    get: NodeType$CurlyGroup_getInstance
  });
  Object.defineProperty(NodeType, 'NamedGroup', {
    get: NodeType$NamedGroup_getInstance
  });
  Object.defineProperty(NodeType, 'Command', {
    get: NodeType$Command_getInstance
  });
  Object.defineProperty(NodeType, 'CommandPart', {
    get: NodeType$CommandPart_getInstance
  });
  Object.defineProperty(NodeType, 'Expression', {
    get: NodeType$Expression_getInstance
  });
  Object.defineProperty(NodeType, 'SubSup', {
    get: NodeType$SubSup_getInstance
  });
  Object.defineProperty(NodeType, 'Parameters', {
    get: NodeType$Parameters_getInstance
  });
  Object.defineProperty(NodeType, 'Comma', {
    get: NodeType$Comma_getInstance
  });
  Object.defineProperty(NodeType, 'Is', {
    get: NodeType$Is_getInstance
  });
  Object.defineProperty(NodeType, 'ColonEquals', {
    get: NodeType$ColonEquals_getInstance
  });
  var package$textalk = package$common.textalk || (package$common.textalk = {});
  package$textalk.NodeType = NodeType;
  package$textalk.Node = Node;
  package$textalk.IsNode = IsNode;
  package$textalk.ColonEqualsNode = ColonEqualsNode;
  package$textalk.CommandPart = CommandPart;
  package$textalk.Command = Command;
  package$textalk.ExpressionNode = ExpressionNode;
  package$textalk.ParametersNode = ParametersNode;
  package$textalk.GroupNode = GroupNode;
  package$textalk.NamedGroupNode = NamedGroupNode;
  package$textalk.SubSupNode = SubSupNode;
  package$textalk.TextNode = TextNode;
  package$textalk.TexTalkToken = TexTalkToken;
  Object.defineProperty(TexTalkTokenType, 'Backslash', {
    get: TexTalkTokenType$Backslash_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'LParen', {
    get: TexTalkTokenType$LParen_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'RParen', {
    get: TexTalkTokenType$RParen_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'LSquare', {
    get: TexTalkTokenType$LSquare_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'RSquare', {
    get: TexTalkTokenType$RSquare_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'LCurly', {
    get: TexTalkTokenType$LCurly_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'RCurly', {
    get: TexTalkTokenType$RCurly_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'Operator', {
    get: TexTalkTokenType$Operator_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'Identifier', {
    get: TexTalkTokenType$Identifier_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'Comma', {
    get: TexTalkTokenType$Comma_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'Period', {
    get: TexTalkTokenType$Period_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'Colon', {
    get: TexTalkTokenType$Colon_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'Underscore', {
    get: TexTalkTokenType$Underscore_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'Caret', {
    get: TexTalkTokenType$Caret_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'ColonEquals', {
    get: TexTalkTokenType$ColonEquals_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'Is', {
    get: TexTalkTokenType$Is_getInstance
  });
  Object.defineProperty(TexTalkTokenType, 'Invalid', {
    get: TexTalkTokenType$Invalid_getInstance
  });
  package$textalk.TexTalkTokenType = TexTalkTokenType;
  package$textalk.TexTalkLexer = TexTalkLexer;
  package$textalk.newTexTalkLexer_61zpoe$ = newTexTalkLexer;
  package$textalk.TexTalkParser = TexTalkParser;
  package$textalk.TexTalkParseResult = TexTalkParseResult;
  package$textalk.newTexTalkParser = newTexTalkParser;
  package$textalk.TexTalkParserImpl = TexTalkParserImpl;
  CLAUSE_VALIDATORS = listOf_0([new ValidationPair(getCallableRef('isAbstraction', function ($receiver, node) {
    return $receiver.isAbstraction_rk66c5$(node);
  }.bind(null, AbstractionNode$Companion_getInstance())), getCallableRef('validate', function ($receiver, node) {
    return $receiver.validate_rk66c5$(node);
  }.bind(null, AbstractionNode$Companion_getInstance()))), new ValidationPair(getCallableRef('isAggregate', function ($receiver, node) {
    return $receiver.isAggregate_rk66c5$(node);
  }.bind(null, AggregateNode$Companion_getInstance())), getCallableRef('validate', function ($receiver, node) {
    return $receiver.validate_rk66c5$(node);
  }.bind(null, AggregateNode$Companion_getInstance()))), new ValidationPair(getCallableRef('isTuple', function ($receiver, node) {
    return $receiver.isTuple_rk66c5$(node);
  }.bind(null, TupleNode$Companion_getInstance())), getCallableRef('validate', function ($receiver, node) {
    return $receiver.validate_rk66c5$(node);
  }.bind(null, TupleNode$Companion_getInstance()))), new ValidationPair(getCallableRef('isAssignment', function ($receiver, node) {
    return $receiver.isAssignment_rk66c5$(node);
  }.bind(null, AssignmentNode$Companion_getInstance())), getCallableRef('validate', function ($receiver, node) {
    return $receiver.validate_rk66c5$(node);
  }.bind(null, AssignmentNode$Companion_getInstance()))), new ValidationPair(getCallableRef('isIdentifier', function ($receiver, node) {
    return $receiver.isIdentifier_rk66c5$(node);
  }.bind(null, Identifier$Companion_getInstance())), getCallableRef('validate', function ($receiver, rawNode) {
    return $receiver.validate_rk66c5$(rawNode);
  }.bind(null, Identifier$Companion_getInstance()))), new ValidationPair(getCallableRef('isStatement', function ($receiver, node) {
    return $receiver.isStatement_rk66c5$(node);
  }.bind(null, Statement$Companion_getInstance())), getCallableRef('validate', function ($receiver, rawNode) {
    return $receiver.validate_rk66c5$(rawNode);
  }.bind(null, Statement$Companion_getInstance()))), new ValidationPair(getCallableRef('isText', function ($receiver, node) {
    return $receiver.isText_rk66c5$(node);
  }.bind(null, Text$Companion_getInstance())), getCallableRef('validate', function ($receiver, rawNode) {
    return $receiver.validate_rk66c5$(rawNode);
  }.bind(null, Text$Companion_getInstance()))), new ValidationPair(getCallableRef('isForGroup', function ($receiver, node) {
    return $receiver.isForGroup_rk66c5$(node);
  }.bind(null, ForGroup$Companion_getInstance())), getCallableRef('validate', function ($receiver, rawNode) {
    return $receiver.validate_rk66c5$(rawNode);
  }.bind(null, ForGroup$Companion_getInstance()))), new ValidationPair(getCallableRef('isExistsGroup', function ($receiver, node) {
    return $receiver.isExistsGroup_rk66c5$(node);
  }.bind(null, ExistsGroup$Companion_getInstance())), getCallableRef('validate', function ($receiver, node) {
    return $receiver.validate_rk66c5$(node);
  }.bind(null, ExistsGroup$Companion_getInstance()))), new ValidationPair(getCallableRef('isNotGroup', function ($receiver, node) {
    return $receiver.isNotGroup_rk66c5$(node);
  }.bind(null, NotGroup$Companion_getInstance())), getCallableRef('validate', function ($receiver, node) {
    return $receiver.validate_rk66c5$(node);
  }.bind(null, NotGroup$Companion_getInstance()))), new ValidationPair(getCallableRef('isOrGroup', function ($receiver, node) {
    return $receiver.isOrGroup_rk66c5$(node);
  }.bind(null, OrGroup$Companion_getInstance())), getCallableRef('validate', function ($receiver, node) {
    return $receiver.validate_rk66c5$(node);
  }.bind(null, OrGroup$Companion_getInstance()))), new ValidationPair(getCallableRef('isIfGroup', function ($receiver, node) {
    return $receiver.isIfGroup_rk66c5$(node);
  }.bind(null, IfGroup$Companion_getInstance())), getCallableRef('validate', function ($receiver, node) {
    return $receiver.validate_rk66c5$(node);
  }.bind(null, IfGroup$Companion_getInstance()))), new ValidationPair(getCallableRef('isIffGroup', function ($receiver, node) {
    return $receiver.isIffGroup_rk66c5$(node);
  }.bind(null, IffGroup$Companion_getInstance())), getCallableRef('validate', function ($receiver, node) {
    return $receiver.validate_rk66c5$(node);
  }.bind(null, IffGroup$Companion_getInstance())))]);
  Kotlin.defineModule('bundle', _);
  return _;
}(typeof bundle === 'undefined' ? {} : bundle, kotlin);
