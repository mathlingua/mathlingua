if (typeof kotlin === 'undefined') {
  throw new Error("Error loading module 'bundle'. Its dependency 'kotlin' was not found. Please, check whether 'kotlin' is loaded prior to 'bundle'.");
}
var bundle = function (_, Kotlin) {
  'use strict';
  var toList = Kotlin.kotlin.collections.toList_7wnvza$;
  var Kind_CLASS = Kotlin.Kind.CLASS;
  var ArrayList_init = Kotlin.kotlin.collections.ArrayList_init_287e2$;
  var copyToArray = Kotlin.kotlin.collections.copyToArray;
  var RuntimeException_init = Kotlin.kotlin.RuntimeException_init_pdl1vj$;
  var RuntimeException = Kotlin.kotlin.RuntimeException;
  var Iterable = Kotlin.kotlin.collections.Iterable;
  var Kind_INTERFACE = Kotlin.Kind.INTERFACE;
  var endsWith = Kotlin.kotlin.text.endsWith_7epoxm$;
  var ensureNotNull = Kotlin.ensureNotNull;
  var toBoxedChar = Kotlin.toBoxedChar;
  var contains = Kotlin.kotlin.text.contains_sgbm27$;
  var Regex_init = Kotlin.kotlin.text.Regex_init_61zpoe$;
  var listOf = Kotlin.kotlin.collections.listOf_mh5how$;
  var StringBuilder_init = Kotlin.kotlin.text.StringBuilder_init;
  var throwCCE = Kotlin.throwCCE;
  var collectionSizeOrDefault = Kotlin.kotlin.collections.collectionSizeOrDefault_ba2ldo$;
  var ArrayList_init_0 = Kotlin.kotlin.collections.ArrayList_init_ww73n8$;
  var Unit = Kotlin.kotlin.Unit;
  var Enum = Kotlin.kotlin.Enum;
  var throwISE = Kotlin.throwISE;
  var equals = Kotlin.equals;
  var getCallableRef = Kotlin.getCallableRef;
  var listOf_0 = Kotlin.kotlin.collections.listOf_i5x0yv$;
  var emptyList = Kotlin.kotlin.collections.emptyList_287e2$;
  var HashMap_init = Kotlin.kotlin.collections.HashMap_init_q3lmfv$;
  var StringBuilder = Kotlin.kotlin.text.StringBuilder;
  var reversed = Kotlin.kotlin.collections.reversed_7wnvza$;
  var last = Kotlin.kotlin.collections.last_2p1efm$;
  var HashSet_init = Kotlin.kotlin.collections.HashSet_init_287e2$;
  var distinct = Kotlin.kotlin.collections.distinct_7wnvza$;
  var Error_init = Kotlin.kotlin.Error_init_pdl1vj$;
  var first = Kotlin.kotlin.collections.first_2p1efm$;
  var Collection = Kotlin.kotlin.collections.Collection;
  var first_0 = Kotlin.kotlin.collections.first_7wnvza$;
  var emptySet = Kotlin.kotlin.collections.emptySet_287e2$;
  var LinkedHashSet_init = Kotlin.kotlin.collections.LinkedHashSet_init_287e2$;
  var filterNotNull = Kotlin.kotlin.collections.filterNotNull_m3lr2h$;
  var toSet = Kotlin.kotlin.collections.toSet_7wnvza$;
  var isBlank = Kotlin.kotlin.text.isBlank_gw00vp$;
  var iterator = Kotlin.kotlin.text.iterator_gw00vp$;
  var unboxChar = Kotlin.unboxChar;
  var LinkedHashMap_init = Kotlin.kotlin.collections.LinkedHashMap_init_q3lmfv$;
  var replace = Kotlin.kotlin.text.replace_680rmw$;
  var sortedWith = Kotlin.kotlin.collections.sortedWith_eknfly$;
  var wrapFunction = Kotlin.wrapFunction;
  var Comparator = Kotlin.kotlin.Comparator;
  ParseError.prototype = Object.create(RuntimeException.prototype);
  ParseError.prototype.constructor = ParseError;
  ValidationSuccess.prototype = Object.create(Validation.prototype);
  ValidationSuccess.prototype.constructor = ValidationSuccess;
  ValidationFailure.prototype = Object.create(Validation.prototype);
  ValidationFailure.prototype.constructor = ValidationFailure;
  ChalkTalkTokenType.prototype = Object.create(Enum.prototype);
  ChalkTalkTokenType.prototype.constructor = ChalkTalkTokenType;
  TupleItem.prototype = Object.create(Phase1Target.prototype);
  TupleItem.prototype.constructor = TupleItem;
  AssignmentRhs.prototype = Object.create(TupleItem.prototype);
  AssignmentRhs.prototype.constructor = AssignmentRhs;
  Phase1Token.prototype = Object.create(AssignmentRhs.prototype);
  Phase1Token.prototype.constructor = Phase1Token;
  Mapping.prototype = Object.create(Phase1Target.prototype);
  Mapping.prototype.constructor = Mapping;
  Group.prototype = Object.create(Phase1Target.prototype);
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
  TexTalkNodeType.prototype = Object.create(Enum.prototype);
  TexTalkNodeType.prototype.constructor = TexTalkNodeType;
  TexTalkTokenType.prototype = Object.create(Enum.prototype);
  TexTalkTokenType.prototype.constructor = TexTalkTokenType;
  function MathLingua() {
  }
  MathLingua.prototype.parse_61zpoe$ = function (input) {
    var tmp$;
    var lexer = newChalkTalkLexer(input);
    var allErrors = ArrayList_init();
    allErrors.addAll_brywnq$(lexer.errors());
    var parser = newChalkTalkParser();
    var tmp$_0 = parser.parse_khrmll$(lexer);
    var root = tmp$_0.component1()
    , errors = tmp$_0.component2();
    allErrors.addAll_brywnq$(errors);
    if (root == null) {
      return new ValidationFailure(allErrors);
    }
    var documentValidation = validateDocument(root);
    if (Kotlin.isType(documentValidation, ValidationSuccess))
      tmp$ = documentValidation;
    else if (Kotlin.isType(documentValidation, ValidationFailure)) {
      allErrors.addAll_brywnq$(documentValidation.errors);
      tmp$ = new ValidationFailure(allErrors);
    }
     else
      tmp$ = Kotlin.noWhenBranchMatched();
    return tmp$;
  };
  MathLingua.prototype.findAllSignatures_mu0sga$ = function (node) {
    return copyToArray(toList(locateAllSignatures(node)));
  };
  MathLingua.prototype.expand_8vvjcc$ = function (doc) {
    return fullExpandComplete(doc);
  };
  MathLingua.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'MathLingua',
    interfaces: []
  };
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
  function Validation() {
  }
  Validation.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Validation',
    interfaces: []
  };
  function ValidationSuccess(value) {
    Validation.call(this);
    this.value = value;
  }
  ValidationSuccess.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ValidationSuccess',
    interfaces: [Validation]
  };
  ValidationSuccess.prototype.component1 = function () {
    return this.value;
  };
  ValidationSuccess.prototype.copy_11rb$ = function (value) {
    return new ValidationSuccess(value === void 0 ? this.value : value);
  };
  ValidationSuccess.prototype.toString = function () {
    return 'ValidationSuccess(value=' + Kotlin.toString(this.value) + ')';
  };
  ValidationSuccess.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.value) | 0;
    return result;
  };
  ValidationSuccess.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.value, other.value))));
  };
  function ValidationFailure(errors) {
    Validation.call(this);
    this.errors = errors;
  }
  ValidationFailure.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ValidationFailure',
    interfaces: [Validation]
  };
  ValidationFailure.prototype.component1 = function () {
    return this.errors;
  };
  ValidationFailure.prototype.copy_hi82q1$ = function (errors) {
    return new ValidationFailure(errors === void 0 ? this.errors : errors);
  };
  ValidationFailure.prototype.toString = function () {
    return 'ValidationFailure(errors=' + Kotlin.toString(this.errors) + ')';
  };
  ValidationFailure.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.errors) | 0;
    return result;
  };
  ValidationFailure.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.errors, other.errors))));
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
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token('=', ChalkTalkTokenType$Equals_getInstance(), line, column));
      }
       else if (c === 40) {
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token('(', ChalkTalkTokenType$LParen_getInstance(), line, column));
      }
       else if (c === 41) {
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token(')', ChalkTalkTokenType$RParen_getInstance(), line, column));
      }
       else if (c === 123) {
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token('{', ChalkTalkTokenType$LCurly_getInstance(), line, column));
      }
       else if (c === 125) {
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token('}', ChalkTalkTokenType$RCurly_getInstance(), line, column));
      }
       else if (c === 58) {
        if (i < this.text_0.length && this.text_0.charCodeAt(i) === 61) {
          ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token(':=', ChalkTalkTokenType$ColonEquals_getInstance(), line, column));
          i = i + 1 | 0;
        }
         else {
          ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token(':', ChalkTalkTokenType$Colon_getInstance(), line, column));
        }
      }
       else if (c === 44) {
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token(',', ChalkTalkTokenType$Comma_getInstance(), line, column));
      }
       else if (c === 46 && i < this.text_0.length && this.text_0.charCodeAt(i) === 32) {
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token('. ', ChalkTalkTokenType$DotSpace_getInstance(), line, column));
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
          ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token('-', ChalkTalkTokenType$Linebreak_getInstance(), line, column));
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
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token('<Indent>', ChalkTalkTokenType$Begin_getInstance(), line, column));
        numOpen = numOpen + 1 | 0;
        var level = levStack.peek();
        if (indentCount <= level) {
          while (numOpen > 0 && !levStack.isEmpty() && indentCount <= levStack.peek()) {
            ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token('<Unindent>', ChalkTalkTokenType$End_getInstance(), line, column));
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
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token(name, ChalkTalkTokenType$Name_getInstance(), line, column));
      }
       else if (this.isNameChar_0(c)) {
        var name_0 = '' + String.fromCharCode(toBoxedChar(c));
        while (i < this.text_0.length && this.isNameChar_0(this.text_0.charCodeAt(i))) {
          tmp$_7 = name_0;
          tmp$_6 = this.text_0;
          tmp$_5 = (tmp$_4 = i, i = tmp$_4 + 1 | 0, tmp$_4);
          name_0 = tmp$_7 + String.fromCharCode(tmp$_6.charCodeAt(tmp$_5));
          column = column + 1 | 0;
        }
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token(name_0, ChalkTalkTokenType$Name_getInstance(), line, column));
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
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token(str, ChalkTalkTokenType$String_getInstance(), line, column));
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
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token(stmt, ChalkTalkTokenType$Statement_getInstance(), line, column));
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
        ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token(id, ChalkTalkTokenType$Id_getInstance(), startLine, startColumn));
      }
       else if (c !== 32) {
        this.errors_0.add_11rb$(new ParseError('Unrecognized character ' + String.fromCharCode(c), line, column));
      }
    }
    while (numOpen > 0) {
      ensureNotNull(this.chalkTalkTokens_0).add_11rb$(new Phase1Token('<Unindent>', ChalkTalkTokenType$End_getInstance(), line, column));
      numOpen = numOpen - 1 | 0;
    }
  };
  ChalkTalkLexerImpl.prototype.isOperatorChar_0 = function (c) {
    return contains('~!@%^&*-+<>\\/=', c);
  };
  ChalkTalkLexerImpl.prototype.isNameChar_0 = function (c) {
    return Regex_init('[$#a-zA-Z0-9]+').matches_6bul2c$(String.fromCharCode(c));
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
  ChalkTalkLexerImpl.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ChalkTalkLexerImpl',
    interfaces: [ChalkTalkLexer]
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
  var INVALID;
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
    this.errors = ArrayList_init();
  }
  ChalkTalkParserImpl$ParserWorker.prototype.root = function () {
    var groups = ArrayList_init();
    while (this.hasNext_0()) {
      var grp = this.group_0();
      if (grp == null)
        break;
      groups.add_11rb$(grp);
    }
    while (this.hasNext_0()) {
      var next = this.next_0();
      this.addError_0("Unrecognized token '" + next.text, next);
    }
    return new Root(groups);
  };
  ChalkTalkParserImpl$ParserWorker.prototype.group_0 = function () {
    if (this.has_0(ChalkTalkTokenType$Linebreak_getInstance())) {
      this.next_0();
    }
    var id = null;
    if (this.has_0(ChalkTalkTokenType$Id_getInstance())) {
      id = this.next_0();
      this.expect_0(ChalkTalkTokenType$Begin_getInstance());
      this.expect_0(ChalkTalkTokenType$End_getInstance());
    }
    var sections = ArrayList_init();
    while (this.hasNext_0()) {
      var sec = this.section_0();
      if (sec == null)
        break;
      sections.add_11rb$(sec);
    }
    return sections.isEmpty() ? null : new Group(sections, id);
  };
  ChalkTalkParserImpl$ParserWorker.prototype.section_0 = function () {
    if (!this.hasHas_0(ChalkTalkTokenType$Name_getInstance(), ChalkTalkTokenType$Colon_getInstance())) {
      return null;
    }
    var name = this.expect_0(ChalkTalkTokenType$Name_getInstance());
    this.expect_0(ChalkTalkTokenType$Colon_getInstance());
    var args = ArrayList_init();
    while (this.hasNext_0() && !this.has_0(ChalkTalkTokenType$Begin_getInstance())) {
      var arg = this.argument_0();
      if (arg == null)
        break;
      args.add_11rb$(arg);
      if (this.hasNext_0() && !this.has_0(ChalkTalkTokenType$Begin_getInstance())) {
        this.expect_0(ChalkTalkTokenType$Comma_getInstance());
      }
    }
    this.expect_0(ChalkTalkTokenType$Begin_getInstance());
    while (this.hasNext_0()) {
      var argList = this.argumentList_0();
      if (argList == null)
        break;
      args.addAll_brywnq$(argList);
    }
    this.expect_0(ChalkTalkTokenType$End_getInstance());
    return new Section(name, args);
  };
  ChalkTalkParserImpl$ParserWorker.prototype.argumentList_0 = function () {
    if (!this.hasNext_0() || !this.has_0(ChalkTalkTokenType$DotSpace_getInstance())) {
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
      while (this.has_0(ChalkTalkTokenType$Comma_getInstance())) {
        this.next_0();
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
      tmp$ = this.next_0();
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
      this.addError_0('Expected a name, abstraction, tuple, aggregate, or assignment');
      return new Argument(INVALID);
    }
    return new Argument(target);
  };
  ChalkTalkParserImpl$ParserWorker.prototype.mapping_0 = function () {
    var tmp$;
    if (!this.hasHas_0(ChalkTalkTokenType$Name_getInstance(), ChalkTalkTokenType$Equals_getInstance())) {
      return null;
    }
    var name = this.next_0();
    var equals = this.next_0();
    if (!this.hasNext_0()) {
      this.addError_0('A = must be followed by an argument', equals);
      tmp$ = INVALID;
    }
     else {
      var maybeRhs = this.next_0();
      if (maybeRhs.type === ChalkTalkTokenType$String_getInstance()) {
        tmp$ = maybeRhs;
      }
       else {
        this.addError_0('The right hand side of a = must be a string', equals);
        tmp$ = INVALID;
      }
    }
    var rhs = tmp$;
    return new Mapping(name, rhs);
  };
  ChalkTalkParserImpl$ParserWorker.prototype.assignment_0 = function () {
    if (!this.hasHas_0(ChalkTalkTokenType$Name_getInstance(), ChalkTalkTokenType$ColonEquals_getInstance())) {
      return null;
    }
    var name = this.next_0();
    var colonEquals = this.next_0();
    var rhs = this.assignmentRhs_0();
    if (rhs == null) {
      this.addError_0('A := must be followed by a argument', colonEquals);
      rhs = INVALID;
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
      if (this.hasNext_0()) {
        var peek = this.next_0();
        this.addError_0('Expected a name, but found ' + peek.text, peek);
      }
       else {
        this.addError_0('Expected a name, but found the end of input');
      }
      return INVALID;
    }
    return this.next_0();
  };
  ChalkTalkParserImpl$ParserWorker.prototype.tuple_0 = function () {
    if (!this.has_0(ChalkTalkTokenType$LParen_getInstance())) {
      return null;
    }
    var items = ArrayList_init();
    var leftParen = this.expect_0(ChalkTalkTokenType$LParen_getInstance());
    while (this.hasNext_0() && !this.has_0(ChalkTalkTokenType$RParen_getInstance())) {
      if (!items.isEmpty()) {
        this.expect_0(ChalkTalkTokenType$Comma_getInstance());
      }
      var item = this.tupleItem_0();
      if (item == null) {
        this.addError_0('Encountered a non-tuple item in a tuple', leftParen);
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
  ChalkTalkParserImpl$ParserWorker.prototype.hasNext_0 = function () {
    return this.chalkTalkLexer_0.hasNext();
  };
  ChalkTalkParserImpl$ParserWorker.prototype.next_0 = function () {
    return this.chalkTalkLexer_0.next();
  };
  ChalkTalkParserImpl$ParserWorker.prototype.addError_0 = function (message, token) {
    if (token === void 0)
      token = null;
    var tmp$, tmp$_0;
    var row = (tmp$ = token != null ? token.row : null) != null ? tmp$ : -1;
    var column = (tmp$_0 = token != null ? token.column : null) != null ? tmp$_0 : -1;
    this.errors.add_11rb$(new ParseError(message, row, column));
  };
  ChalkTalkParserImpl$ParserWorker.prototype.nameList_0 = function (stopType) {
    var names = ArrayList_init();
    while (this.hasNext_0() && !this.has_0(stopType)) {
      var comma = null;
      if (!names.isEmpty()) {
        comma = this.expect_0(ChalkTalkTokenType$Comma_getInstance());
      }
      if (!this.hasNext_0()) {
        this.addError_0('Expected a name to follow a comma', comma);
        break;
      }
      var tok = this.next_0();
      if (tok.type === ChalkTalkTokenType$Name_getInstance()) {
        names.add_11rb$(tok);
      }
       else {
        this.addError_0("Expected a name but found '" + tok.text + "'", tok);
      }
    }
    return names;
  };
  ChalkTalkParserImpl$ParserWorker.prototype.has_0 = function (type) {
    return this.hasNext_0() && this.chalkTalkLexer_0.peek().type === type;
  };
  ChalkTalkParserImpl$ParserWorker.prototype.hasHas_0 = function (type, thenType) {
    return this.has_0(type) && this.chalkTalkLexer_0.hasNextNext() && this.chalkTalkLexer_0.peekPeek().type === thenType;
  };
  ChalkTalkParserImpl$ParserWorker.prototype.expect_0 = function (type) {
    var tmp$;
    if (!this.hasNext_0() || this.chalkTalkLexer_0.peek().type !== type) {
      if (this.hasNext_0()) {
        tmp$ = this.chalkTalkLexer_0.peek();
      }
       else {
        tmp$ = INVALID;
      }
      var peek = tmp$;
      this.addError_0('Expected a token of type ' + type + ' but found ' + peek.type, peek);
      return INVALID;
    }
    return this.next_0();
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
  function Phase1Node() {
  }
  Phase1Node.$metadata$ = {
    kind: Kind_INTERFACE,
    simpleName: 'Phase1Node',
    interfaces: []
  };
  function Root(groups) {
    this.groups = groups;
  }
  Root.prototype.forEach_t0jmzf$ = function (fn) {
    var tmp$;
    tmp$ = this.groups.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  Root.prototype.print_0 = function (buffer) {
    var tmp$;
    tmp$ = this.groups.iterator();
    while (tmp$.hasNext()) {
      var grp = tmp$.next();
      grp.print_yrfq27$(buffer, 0, false);
    }
  };
  Root.prototype.toCode = function () {
    var buffer = StringBuilder_init();
    this.print_0(buffer);
    return buffer.toString();
  };
  Root.prototype.resolve = function () {
    return this;
  };
  Root.prototype.transform_w8pxcw$ = function (transformer) {
    var $receiver = this.groups;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      var tmp$_0;
      destination.add_11rb$(Kotlin.isType(tmp$_0 = item.transform_w8pxcw$(transformer), Group) ? tmp$_0 : throwCCE());
    }
    return transformer(new Root(destination));
  };
  Root.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Root',
    interfaces: [Phase1Node]
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
  Argument.prototype.forEach_t0jmzf$ = function (fn) {
    fn(this.chalkTalkTarget);
  };
  Argument.prototype.print_rg88dk$ = function (buffer, level) {
    var tmp$;
    tmp$ = this.chalkTalkTarget;
    if (Kotlin.isType(tmp$, Group))
      this.chalkTalkTarget.print_yrfq27$(buffer, level, true);
    else if (Kotlin.isType(tmp$, Phase1Token)) {
      buffer.append_gw00v9$(buildIndent(level, true));
      buffer.append_gw00v9$(this.chalkTalkTarget.text);
      buffer.append_gw00v9$('\n');
    }
     else if (Kotlin.isType(tmp$, Abstraction)) {
      buffer.append_gw00v9$(buildIndent(level, true));
      buffer.append_gw00v9$(this.chalkTalkTarget.toCode());
      buffer.append_gw00v9$('\n');
    }
     else if (Kotlin.isType(tmp$, Aggregate)) {
      buffer.append_gw00v9$(buildIndent(level, true));
      buffer.append_gw00v9$(this.chalkTalkTarget.toCode());
      buffer.append_gw00v9$('\n');
    }
     else if (Kotlin.isType(tmp$, Assignment)) {
      buffer.append_gw00v9$(buildIndent(level, true));
      buffer.append_gw00v9$(this.chalkTalkTarget.toCode());
      buffer.append_gw00v9$('\n');
    }
     else if (Kotlin.isType(tmp$, Mapping)) {
      buffer.append_gw00v9$(buildIndent(level, true));
      buffer.append_gw00v9$(this.chalkTalkTarget.toCode());
      buffer.append_gw00v9$('\n');
    }
     else if (Kotlin.isType(tmp$, Tuple)) {
      buffer.append_gw00v9$(buildIndent(level, true));
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
  Argument.prototype.transform_w8pxcw$ = function (transformer) {
    var tmp$;
    return transformer(new Argument(Kotlin.isType(tmp$ = this.chalkTalkTarget.transform_w8pxcw$(transformer), Phase1Target) ? tmp$ : throwCCE()));
  };
  Argument.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Argument',
    interfaces: [Phase1Node]
  };
  Argument.prototype.component1 = function () {
    return this.chalkTalkTarget;
  };
  Argument.prototype.copy_g40oyj$ = function (chalkTalkTarget) {
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
  Section.prototype.forEach_t0jmzf$ = function (fn) {
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
    buffer.append_gw00v9$(buildIndent(level, fromArg));
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
  Section.prototype.transform_w8pxcw$ = function (transformer) {
    var tmp$, tmp$_0;
    tmp$_0 = Kotlin.isType(tmp$ = this.name.transform_w8pxcw$(transformer), Phase1Token) ? tmp$ : throwCCE();
    var $receiver = this.args;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$_1;
    tmp$_1 = $receiver.iterator();
    while (tmp$_1.hasNext()) {
      var item = tmp$_1.next();
      var tmp$_2;
      destination.add_11rb$(Kotlin.isType(tmp$_2 = item.transform_w8pxcw$(transformer), Argument) ? tmp$_2 : throwCCE());
    }
    return transformer(new Section(tmp$_0, destination));
  };
  Section.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Section',
    interfaces: [Phase1Node]
  };
  Section.prototype.component1 = function () {
    return this.name;
  };
  Section.prototype.component2 = function () {
    return this.args;
  };
  Section.prototype.copy_f42m9b$ = function (name, args) {
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
  function max(val1, val2) {
    return val1 >= val2 ? val1 : val2;
  }
  function buildIndent(level, isArg) {
    var buffer = StringBuilder_init();
    var numSpaces = isArg ? 2 * max(level - 1 | 0, 0) | 0 : 2 * level | 0;
    for (var i = 0; i < numSpaces; i++) {
      buffer.append_s8itvh$(32);
    }
    if (isArg) {
      buffer.append_gw00v9$('. ');
    }
    return buffer.toString();
  }
  function getIndent(size) {
    var buffer = StringBuilder_init();
    for (var i = 0; i < size; i++) {
      buffer.append_s8itvh$(32);
    }
    return buffer.toString();
  }
  function getRow$lambda(closure$rowResult) {
    return function (it) {
      if (closure$rowResult.v === -1) {
        var row = getRow(it);
        if (row >= 0) {
          closure$rowResult.v = row;
        }
      }
      return Unit;
    };
  }
  function getRow(node) {
    if (Kotlin.isType(node, Phase1Token)) {
      return node.row;
    }
    var rowResult = {v: -1};
    node.forEach_t0jmzf$(getRow$lambda(rowResult));
    return rowResult.v;
  }
  function getColumn$lambda(closure$colResult) {
    return function (it) {
      if (closure$colResult.v === -1) {
        var col = getColumn(it);
        if (col >= 0) {
          closure$colResult.v = col;
        }
      }
      return Unit;
    };
  }
  function getColumn(node) {
    if (Kotlin.isType(node, Phase1Token)) {
      return node.column;
    }
    var colResult = {v: -1};
    node.forEach_t0jmzf$(getColumn$lambda(colResult));
    return colResult.v;
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
  function Phase1Target() {
  }
  Phase1Target.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Phase1Target',
    interfaces: [Phase1Node]
  };
  function TupleItem() {
    Phase1Target.call(this);
  }
  TupleItem.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'TupleItem',
    interfaces: [Phase1Target]
  };
  function AssignmentRhs() {
    TupleItem.call(this);
  }
  AssignmentRhs.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'AssignmentRhs',
    interfaces: [TupleItem]
  };
  function Phase1Token(text, type, row, column) {
    AssignmentRhs.call(this);
    this.text = text;
    this.type = type;
    this.row = row;
    this.column = column;
  }
  Phase1Token.prototype.forEach_t0jmzf$ = function (fn) {
  };
  Phase1Token.prototype.toCode = function () {
    return this.text;
  };
  Phase1Token.prototype.resolve = function () {
    return this;
  };
  Phase1Token.prototype.transform_w8pxcw$ = function (transformer) {
    return transformer(this);
  };
  Phase1Token.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Phase1Token',
    interfaces: [AssignmentRhs]
  };
  Phase1Token.prototype.component1 = function () {
    return this.text;
  };
  Phase1Token.prototype.component2 = function () {
    return this.type;
  };
  Phase1Token.prototype.component3 = function () {
    return this.row;
  };
  Phase1Token.prototype.component4 = function () {
    return this.column;
  };
  Phase1Token.prototype.copy_m2738k$ = function (text, type, row, column) {
    return new Phase1Token(text === void 0 ? this.text : text, type === void 0 ? this.type : type, row === void 0 ? this.row : row, column === void 0 ? this.column : column);
  };
  Phase1Token.prototype.toString = function () {
    return 'Phase1Token(text=' + Kotlin.toString(this.text) + (', type=' + Kotlin.toString(this.type)) + (', row=' + Kotlin.toString(this.row)) + (', column=' + Kotlin.toString(this.column)) + ')';
  };
  Phase1Token.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.text) | 0;
    result = result * 31 + Kotlin.hashCode(this.type) | 0;
    result = result * 31 + Kotlin.hashCode(this.row) | 0;
    result = result * 31 + Kotlin.hashCode(this.column) | 0;
    return result;
  };
  Phase1Token.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.text, other.text) && Kotlin.equals(this.type, other.type) && Kotlin.equals(this.row, other.row) && Kotlin.equals(this.column, other.column)))));
  };
  function Mapping(lhs, rhs) {
    Phase1Target.call(this);
    this.lhs = lhs;
    this.rhs = rhs;
  }
  Mapping.prototype.forEach_t0jmzf$ = function (fn) {
    fn(this.lhs);
    fn(this.rhs);
  };
  Mapping.prototype.toCode = function () {
    return this.lhs.toCode() + ' = ' + this.rhs.toCode();
  };
  Mapping.prototype.resolve = function () {
    return this;
  };
  Mapping.prototype.transform_w8pxcw$ = function (transformer) {
    var tmp$, tmp$_0;
    return transformer(new Mapping(Kotlin.isType(tmp$ = this.lhs.transform_w8pxcw$(transformer), Phase1Token) ? tmp$ : throwCCE(), Kotlin.isType(tmp$_0 = this.rhs.transform_w8pxcw$(transformer), Phase1Token) ? tmp$_0 : throwCCE()));
  };
  Mapping.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Mapping',
    interfaces: [Phase1Target]
  };
  Mapping.prototype.component1 = function () {
    return this.lhs;
  };
  Mapping.prototype.component2 = function () {
    return this.rhs;
  };
  Mapping.prototype.copy_t0oob0$ = function (lhs, rhs) {
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
    Phase1Target.call(this);
    this.sections = sections;
    this.id = id;
  }
  Group.prototype.forEach_t0jmzf$ = function (fn) {
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
  Group.prototype.transform_w8pxcw$ = function (transformer) {
    var tmp$, tmp$_0;
    var $receiver = this.sections;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$_1;
    tmp$_1 = $receiver.iterator();
    while (tmp$_1.hasNext()) {
      var item = tmp$_1.next();
      var tmp$_2;
      destination.add_11rb$(Kotlin.isType(tmp$_2 = item.transform_w8pxcw$(transformer), Section) ? tmp$_2 : throwCCE());
    }
    return transformer(new Group(destination, Kotlin.isType(tmp$_0 = (tmp$ = this.id) != null ? tmp$.transform_w8pxcw$(transformer) : null, Phase1Token) ? tmp$_0 : throwCCE()));
  };
  Group.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Group',
    interfaces: [Phase1Target]
  };
  Group.prototype.component1 = function () {
    return this.sections;
  };
  Group.prototype.component2 = function () {
    return this.id;
  };
  Group.prototype.copy_r7lnki$ = function (sections, id) {
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
  Assignment.prototype.forEach_t0jmzf$ = function (fn) {
    fn(this.lhs);
    fn(this.rhs);
  };
  Assignment.prototype.toCode = function () {
    return this.lhs.toCode() + ' := ' + this.rhs.toCode();
  };
  Assignment.prototype.resolve = function () {
    return this;
  };
  Assignment.prototype.transform_w8pxcw$ = function (transformer) {
    var tmp$, tmp$_0;
    return transformer(new Assignment(Kotlin.isType(tmp$ = this.lhs.transform_w8pxcw$(transformer), Phase1Token) ? tmp$ : throwCCE(), Kotlin.isType(tmp$_0 = this.rhs.transform_w8pxcw$(transformer), Phase1Token) ? tmp$_0 : throwCCE()));
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
  Assignment.prototype.copy_71r553$ = function (lhs, rhs) {
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
  Tuple.prototype.forEach_t0jmzf$ = function (fn) {
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
  Tuple.prototype.transform_w8pxcw$ = function (transformer) {
    var $receiver = this.items;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      var tmp$_0;
      destination.add_11rb$(Kotlin.isType(tmp$_0 = item.transform_w8pxcw$(transformer), TupleItem) ? tmp$_0 : throwCCE());
    }
    return transformer(new Tuple(destination));
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
  Abstraction.prototype.forEach_t0jmzf$ = function (fn) {
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
    tmp$ = this.params;
    for (var i = 0; i !== tmp$.size; ++i) {
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
  Abstraction.prototype.transform_w8pxcw$ = function (transformer) {
    var tmp$, tmp$_0;
    tmp$_0 = Kotlin.isType(tmp$ = this.name.transform_w8pxcw$(transformer), Phase1Token) ? tmp$ : throwCCE();
    var $receiver = this.params;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$_1;
    tmp$_1 = $receiver.iterator();
    while (tmp$_1.hasNext()) {
      var item = tmp$_1.next();
      var tmp$_2;
      destination.add_11rb$(Kotlin.isType(tmp$_2 = item.transform_w8pxcw$(transformer), Phase1Token) ? tmp$_2 : throwCCE());
    }
    return transformer(new Abstraction(tmp$_0, destination));
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
  Abstraction.prototype.copy_bymma5$ = function (name, params) {
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
  Aggregate.prototype.forEach_t0jmzf$ = function (fn) {
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
    tmp$ = this.params;
    for (var i = 0; i !== tmp$.size; ++i) {
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
  Aggregate.prototype.transform_w8pxcw$ = function (transformer) {
    var $receiver = this.params;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      var tmp$_0;
      destination.add_11rb$(Kotlin.isType(tmp$_0 = item.transform_w8pxcw$(transformer), Phase1Token) ? tmp$_0 : throwCCE());
    }
    return transformer(new Aggregate(destination));
  };
  Aggregate.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Aggregate',
    interfaces: [AssignmentRhs]
  };
  Aggregate.prototype.component1 = function () {
    return this.params;
  };
  Aggregate.prototype.copy_ydyqai$ = function (params) {
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
    tmp$ = this.mappings;
    for (var i = 0; i !== tmp$.size; ++i) {
      builder.append_gw00v9$(this.mappings.get_za3lpa$(i).toCode_eltk6l$(true, indent + 2 | 0));
      if (i !== (this.mappings.size - 1 | 0)) {
        builder.append_s8itvh$(10);
      }
    }
    return builder.toString();
  };
  AliasSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var $receiver = this.mappings;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      var tmp$_0;
      destination.add_11rb$(Kotlin.isType(tmp$_0 = item.transform_nrl0ww$(chalkTransformer), MappingNode) ? tmp$_0 : throwCCE());
    }
    return chalkTransformer(new AliasSection(destination));
  };
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
  function validateAliasSection(section) {
    var tmp$, tmp$_0;
    if (!equals(section.name.text, 'Alias')) {
      return new ValidationFailure(listOf(new ParseError("Expected a 'Alias' but found '" + section.name.text + "'", getRow(section), getColumn(section))));
    }
    var errors = ArrayList_init();
    var mappings = ArrayList_init();
    tmp$ = section.args.iterator();
    while (tmp$.hasNext()) {
      var arg = tmp$.next();
      var validation = validateMappingNode(arg);
      if (Kotlin.isType(validation, ValidationSuccess))
        mappings.add_11rb$(validation.value);
      else if (Kotlin.isType(validation, ValidationFailure))
        errors.addAll_brywnq$(validation.errors);
      else
        Kotlin.noWhenBranchMatched();
    }
    if (!errors.isEmpty()) {
      tmp$_0 = new ValidationFailure(errors);
    }
     else {
      tmp$_0 = new ValidationSuccess(new AliasSection(mappings));
    }
    return tmp$_0;
  }
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
  ValidationPair.prototype.copy_h3rwey$ = function (matches, validate) {
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
  }
  Clause.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Clause',
    interfaces: [Phase2Node]
  };
  function validateClause(rawNode) {
    var tmp$, tmp$_0;
    var node = rawNode.resolve();
    tmp$ = CLAUSE_VALIDATORS.iterator();
    while (tmp$.hasNext()) {
      var pair = tmp$.next();
      if (pair.matches(node)) {
        var validation = pair.validate(node);
        if (Kotlin.isType(validation, ValidationSuccess))
          tmp$_0 = new ValidationSuccess(validation.value);
        else if (Kotlin.isType(validation, ValidationFailure))
          tmp$_0 = new ValidationFailure(validation.errors);
        else
          tmp$_0 = Kotlin.noWhenBranchMatched();
        return tmp$_0;
      }
    }
    return new ValidationFailure(listOf(new ParseError('Expected a Target', getRow(node), getColumn(node))));
  }
  function Target() {
    Clause.call(this);
  }
  Target.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Target',
    interfaces: [Clause]
  };
  function AbstractionNode(abstraction) {
    Target.call(this);
    this.abstraction = abstraction;
  }
  AbstractionNode.prototype.forEach_ye21ev$ = function (fn) {
  };
  AbstractionNode.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode(isArg, indent, this.abstraction);
  };
  AbstractionNode.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    return chalkTransformer(this);
  };
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
  function isAbstraction(node) {
    return Kotlin.isType(node, Abstraction);
  }
  function validateAbstractionNode$lambda(it) {
    var tmp$;
    return Kotlin.isType(tmp$ = it, Abstraction) ? tmp$ : null;
  }
  function validateAbstractionNode$lambda_0(it) {
    return new AbstractionNode(it);
  }
  function validateAbstractionNode(node) {
    return validateWrappedNode(node, 'AbstractionNode', validateAbstractionNode$lambda, validateAbstractionNode$lambda_0);
  }
  function AggregateNode(aggregate) {
    Target.call(this);
    this.aggregate = aggregate;
  }
  AggregateNode.prototype.forEach_ye21ev$ = function (fn) {
  };
  AggregateNode.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode(isArg, indent, this.aggregate);
  };
  AggregateNode.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    return chalkTransformer(this);
  };
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
  function isAggregate(node) {
    return Kotlin.isType(node, Aggregate);
  }
  function validateAggregateNode$lambda(it) {
    var tmp$;
    return Kotlin.isType(tmp$ = it, Aggregate) ? tmp$ : null;
  }
  function validateAggregateNode$lambda_0(it) {
    return new AggregateNode(it);
  }
  function validateAggregateNode(node) {
    return validateWrappedNode(node, 'AggregateNode', validateAggregateNode$lambda, validateAggregateNode$lambda_0);
  }
  function TupleNode(tuple) {
    Target.call(this);
    this.tuple = tuple;
  }
  TupleNode.prototype.forEach_ye21ev$ = function (fn) {
  };
  TupleNode.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode(isArg, indent, this.tuple);
  };
  TupleNode.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    return chalkTransformer(this);
  };
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
  function isTuple(node) {
    return Kotlin.isType(node, Tuple);
  }
  function validateTupleNode$lambda(it) {
    var tmp$;
    return Kotlin.isType(tmp$ = it, Tuple) ? tmp$ : null;
  }
  function validateTupleNode$lambda_0(it) {
    return new TupleNode(it);
  }
  function validateTupleNode(node) {
    return validateWrappedNode(node, 'TupleNode', validateTupleNode$lambda, validateTupleNode$lambda_0);
  }
  function AssignmentNode(assignment) {
    Target.call(this);
    this.assignment = assignment;
  }
  AssignmentNode.prototype.forEach_ye21ev$ = function (fn) {
  };
  AssignmentNode.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode(isArg, indent, this.assignment);
  };
  AssignmentNode.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    return chalkTransformer(this);
  };
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
  function isAssignment(node) {
    return Kotlin.isType(node, Assignment);
  }
  function validateAssignmentNode$lambda(it) {
    var tmp$;
    return Kotlin.isType(tmp$ = it, Assignment) ? tmp$ : null;
  }
  function validateAssignmentNode(node) {
    return validateWrappedNode(node, 'AssignmentNode', validateAssignmentNode$lambda, getCallableRef('AssignmentNode', function (assignment) {
      return new AssignmentNode(assignment);
    }));
  }
  function MappingNode(mapping) {
    this.mapping = mapping;
  }
  MappingNode.prototype.forEach_ye21ev$ = function (fn) {
  };
  MappingNode.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode(isArg, indent, this.mapping);
  };
  MappingNode.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    return chalkTransformer(this);
  };
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
  function isMapping(node) {
    return Kotlin.isType(node, Mapping);
  }
  function validateMappingNode$lambda(it) {
    var tmp$;
    return Kotlin.isType(tmp$ = it, Mapping) ? tmp$ : null;
  }
  function validateMappingNode(node) {
    return validateWrappedNode(node, 'MappingNode', validateMappingNode$lambda, getCallableRef('MappingNode', function (mapping) {
      return new MappingNode(mapping);
    }));
  }
  function Identifier(name) {
    Target.call(this);
    this.name = name;
  }
  Identifier.prototype.forEach_ye21ev$ = function (fn) {
  };
  Identifier.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return indentedString(isArg, indent, this.name);
  };
  Identifier.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    return chalkTransformer(this);
  };
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
  function isIdentifier(node) {
    return Kotlin.isType(node, Phase1Token) && node.type === ChalkTalkTokenType$Name_getInstance();
  }
  function validateIdentifier(rawNode) {
    var node = rawNode.resolve();
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Phase1Token)) {
      errors.add_11rb$(new ParseError('Cannot convert to a ChalkTalkToken', getRow(node), getColumn(node)));
      return new ValidationFailure(errors);
    }
    var text = node.component1()
    , type = node.component2()
    , row = node.component3()
    , column = node.component4();
    if (type !== ChalkTalkTokenType$Name_getInstance()) {
      errors.add_11rb$(new ParseError('A token of type ' + type + ' is not an identifier', row, column));
      return new ValidationFailure(errors);
    }
    return new ValidationSuccess(new Identifier(text));
  }
  function Statement(text, texTalkRoot) {
    Clause.call(this);
    this.text = text;
    this.texTalkRoot = texTalkRoot;
  }
  Statement.prototype.forEach_ye21ev$ = function (fn) {
  };
  Statement.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return indentedString(isArg, indent, "'" + this.text + "'");
  };
  Statement.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    return chalkTransformer(this);
  };
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
  Statement.prototype.copy_qe3h67$ = function (text, texTalkRoot) {
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
  function isStatement(node) {
    return Kotlin.isType(node, Phase1Token) && node.type === ChalkTalkTokenType$Statement_getInstance();
  }
  function validateStatement(rawNode) {
    var tmp$, tmp$_0;
    var node = rawNode.resolve();
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Phase1Token)) {
      errors.add_11rb$(new ParseError('Cannot convert a to a ChalkTalkToken', getRow(node), getColumn(node)));
    }
    var tmp$_1 = Kotlin.isType(tmp$ = node, Phase1Token) ? tmp$ : throwCCE();
    var rawText = tmp$_1.component1()
    , type = tmp$_1.component2()
    , row = tmp$_1.component3()
    , column = tmp$_1.component4();
    if (type !== ChalkTalkTokenType$Statement_getInstance()) {
      errors.add_11rb$(new ParseError('Cannot convert a ' + node.toCode() + ' to a Statement', row, column));
      return new ValidationFailure(errors);
    }
    var endIndex = rawText.length - 1 | 0;
    var text = rawText.substring(1, endIndex);
    var texTalkErrors = ArrayList_init();
    var lexer = newTexTalkLexer(text);
    texTalkErrors.addAll_brywnq$(lexer.errors);
    var parser = newTexTalkParser();
    var result = parser.parse_2mg13h$(lexer);
    texTalkErrors.addAll_brywnq$(result.errors);
    if (texTalkErrors.isEmpty()) {
      tmp$_0 = new ValidationSuccess(result.root);
    }
     else {
      tmp$_0 = new ValidationFailure(texTalkErrors);
    }
    var validation = tmp$_0;
    return new ValidationSuccess(new Statement(text, validation));
  }
  function Text(text) {
    Clause.call(this);
    this.text = text;
  }
  Text.prototype.forEach_ye21ev$ = function (fn) {
  };
  Text.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return indentedString(isArg, indent, this.text);
  };
  Text.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    return chalkTransformer(this);
  };
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
  function isText(node) {
    return Kotlin.isType(node, Phase1Token) && node.type === ChalkTalkTokenType$String_getInstance();
  }
  function validateText(rawNode) {
    var tmp$;
    var node = rawNode.resolve();
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Phase1Token)) {
      errors.add_11rb$(new ParseError('Cannot convert a to a ChalkTalkToken', getRow(node), getColumn(node)));
    }
    var tmp$_0 = Kotlin.isType(tmp$ = node, Phase1Token) ? tmp$ : throwCCE();
    var text = tmp$_0.component1()
    , type = tmp$_0.component2()
    , row = tmp$_0.component3()
    , column = tmp$_0.component4();
    if (type !== ChalkTalkTokenType$String_getInstance()) {
      errors.add_11rb$(new ParseError('Cannot convert a ' + node.toCode() + ' to Text', row, column));
      return new ValidationFailure(errors);
    }
    return new ValidationSuccess(new Text(text));
  }
  function ExistsGroup(existsSection, suchThatSection) {
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
  ExistsGroup.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$, tmp$_0;
    return chalkTransformer(new ExistsGroup(Kotlin.isType(tmp$ = this.existsSection.transform_nrl0ww$(chalkTransformer), ExistsSection) ? tmp$ : throwCCE(), Kotlin.isType(tmp$_0 = this.suchThatSection.transform_nrl0ww$(chalkTransformer), SuchThatSection) ? tmp$_0 : throwCCE()));
  };
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
  function isExistsGroup(node) {
    return firstSectionMatchesName(node, 'exists');
  }
  function validateExistsGroup(node) {
    return validateDoubleSectionGroup(node, 'exists', getCallableRef('validateExistsSection', function (node) {
      return validateExistsSection(node);
    }), 'suchThat', getCallableRef('validateSuchThatSection', function (node) {
      return validateSuchThatSection(node);
    }), getCallableRef('ExistsGroup', function (existsSection, suchThatSection) {
      return new ExistsGroup(existsSection, suchThatSection);
    }));
  }
  function IfGroup(ifSection, thenSection) {
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
  IfGroup.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$, tmp$_0;
    return chalkTransformer(new IfGroup(Kotlin.isType(tmp$ = this.ifSection.transform_nrl0ww$(chalkTransformer), IfSection) ? tmp$ : throwCCE(), Kotlin.isType(tmp$_0 = this.thenSection.transform_nrl0ww$(chalkTransformer), ThenSection) ? tmp$_0 : throwCCE()));
  };
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
  function isIfGroup(node) {
    return firstSectionMatchesName(node, 'if');
  }
  function validateIfGroup(node) {
    return validateDoubleSectionGroup(node, 'if', getCallableRef('validateIfSection', function (node) {
      return validateIfSection(node);
    }), 'then', getCallableRef('validateThenSection', function (node) {
      return validateThenSection(node);
    }), getCallableRef('IfGroup', function (ifSection, thenSection) {
      return new IfGroup(ifSection, thenSection);
    }));
  }
  function IffGroup(iffSection, thenSection) {
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
  IffGroup.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$, tmp$_0;
    return chalkTransformer(new IffGroup(Kotlin.isType(tmp$ = this.iffSection.transform_nrl0ww$(chalkTransformer), IffSection) ? tmp$ : throwCCE(), Kotlin.isType(tmp$_0 = this.thenSection.transform_nrl0ww$(chalkTransformer), ThenSection) ? tmp$_0 : throwCCE()));
  };
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
  function isIffGroup(node) {
    return firstSectionMatchesName(node, 'iff');
  }
  function validateIffGroup(node) {
    return validateDoubleSectionGroup(node, 'iff', getCallableRef('validateIffSection', function (node) {
      return validateIffSection(node);
    }), 'then', getCallableRef('validateThenSection', function (node) {
      return validateThenSection(node);
    }), getCallableRef('IffGroup', function (iffSection, thenSection) {
      return new IffGroup(iffSection, thenSection);
    }));
  }
  function ForGroup(forSection, whereSection, thenSection) {
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
  ForGroup.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$, tmp$_0, tmp$_1, tmp$_2;
    return chalkTransformer(new ForGroup(Kotlin.isType(tmp$ = this.forSection.transform_nrl0ww$(chalkTransformer), ForSection) ? tmp$ : throwCCE(), (tmp$_1 = (tmp$_0 = this.whereSection) != null ? tmp$_0.transform_nrl0ww$(chalkTransformer) : null) == null || Kotlin.isType(tmp$_1, WhereSection) ? tmp$_1 : throwCCE(), Kotlin.isType(tmp$_2 = this.thenSection.transform_nrl0ww$(chalkTransformer), ThenSection) ? tmp$_2 : throwCCE()));
  };
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
  function isForGroup(node) {
    return firstSectionMatchesName(node, 'for');
  }
  function validateForGroup(rawNode) {
    var tmp$;
    var node = rawNode.resolve();
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Group)) {
      errors.add_11rb$(new ParseError('Expected a Group', getRow(node), getColumn(node)));
      return new ValidationFailure(errors);
    }
    var sections = node.component1();
    var sectionMap;
    try {
      sectionMap = identifySections(sections, ['for', 'where?', 'then']);
    }
     catch (e) {
      if (Kotlin.isType(e, ParseError)) {
        errors.add_11rb$(new ParseError(e.message, e.row, e.column));
        return new ValidationFailure(errors);
      }
       else
        throw e;
    }
    var forSection = null;
    var forNode = sectionMap.get_11rb$('for');
    var forEvaluation = validateForSection(ensureNotNull(forNode));
    if (Kotlin.isType(forEvaluation, ValidationSuccess))
      forSection = forEvaluation.value;
    else if (Kotlin.isType(forEvaluation, ValidationFailure))
      errors.addAll_brywnq$(forEvaluation.errors);
    else
      Kotlin.noWhenBranchMatched();
    var whereSection = null;
    if (sectionMap.containsKey_11rb$('where')) {
      var where = sectionMap.get_11rb$('where');
      var whereValidation = validateWhereSection(ensureNotNull(where));
      if (Kotlin.isType(whereValidation, ValidationSuccess))
        whereSection = whereValidation.value;
      else if (Kotlin.isType(whereValidation, ValidationFailure))
        errors.addAll_brywnq$(whereValidation.errors);
      else
        Kotlin.noWhenBranchMatched();
    }
    var thenSection = null;
    var then = sectionMap.get_11rb$('then');
    var thenValidation = validateThenSection(ensureNotNull(then));
    if (Kotlin.isType(thenValidation, ValidationSuccess))
      thenSection = thenValidation.value;
    else if (Kotlin.isType(thenValidation, ValidationFailure))
      errors.addAll_brywnq$(thenValidation.errors);
    else
      Kotlin.noWhenBranchMatched();
    if (!errors.isEmpty()) {
      tmp$ = new ValidationFailure(errors);
    }
     else
      tmp$ = new ValidationSuccess(new ForGroup(ensureNotNull(forSection), whereSection, ensureNotNull(thenSection)));
    return tmp$;
  }
  function NotGroup(notSection) {
    Clause.call(this);
    this.notSection = notSection;
  }
  NotGroup.prototype.forEach_ye21ev$ = function (fn) {
    fn(this.notSection);
  };
  NotGroup.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return this.notSection.toCode_eltk6l$(isArg, indent);
  };
  NotGroup.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new NotGroup(Kotlin.isType(tmp$ = this.notSection.transform_nrl0ww$(chalkTransformer), NotSection) ? tmp$ : throwCCE()));
  };
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
  function isNotGroup(node) {
    return firstSectionMatchesName(node, 'not');
  }
  function validateNotGroup(node) {
    return validateSingleSectionGroup(node, 'not', getCallableRef('NotGroup', function (notSection) {
      return new NotGroup(notSection);
    }), getCallableRef('validateNotSection', function (node) {
      return validateNotSection(node);
    }));
  }
  function OrGroup(orSection) {
    Clause.call(this);
    this.orSection = orSection;
  }
  OrGroup.prototype.forEach_ye21ev$ = function (fn) {
    fn(this.orSection);
  };
  OrGroup.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return this.orSection.toCode_eltk6l$(isArg, indent);
  };
  OrGroup.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new OrGroup(Kotlin.isType(tmp$ = this.orSection.transform_nrl0ww$(chalkTransformer), OrSection) ? tmp$ : throwCCE()));
  };
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
  function isOrGroup(node) {
    return firstSectionMatchesName(node, 'or');
  }
  function validateOrGroup(node) {
    return validateSingleSectionGroup(node, 'or', getCallableRef('OrGroup', function (orSection) {
      return new OrGroup(orSection);
    }), getCallableRef('validateOrSection', function (node) {
      return validateOrSection(node);
    }));
  }
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
      errors.add_11rb$(new ParseError('Expected a Group', getRow(node), getColumn(node)));
      return new ValidationFailure(errors);
    }
    var sections = node.component1();
    var sectionMap;
    try {
      sectionMap = identifySections(sections, [sectionName]);
    }
     catch (e) {
      if (Kotlin.isType(e, ParseError)) {
        errors.add_11rb$(new ParseError(e.message, e.row, e.column));
        return new ValidationFailure(errors);
      }
       else
        throw e;
    }
    var section = null;
    var sect = sectionMap.get_11rb$(sectionName);
    var validation = validateSection(ensureNotNull(sect));
    if (Kotlin.isType(validation, ValidationSuccess))
      section = validation.value;
    else if (Kotlin.isType(validation, ValidationFailure))
      errors.addAll_brywnq$(validation.errors);
    else
      Kotlin.noWhenBranchMatched();
    if (!errors.isEmpty()) {
      tmp$ = new ValidationFailure(errors);
    }
     else
      tmp$ = new ValidationSuccess(buildGroup(ensureNotNull(section)));
    return tmp$;
  }
  function validateDoubleSectionGroup(rawNode, section1Name, validateSection1, section2Name, validateSection2, buildGroup) {
    var tmp$;
    var node = rawNode.resolve();
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Group)) {
      errors.add_11rb$(new ParseError('Expected a Group', getRow(node), getColumn(node)));
      return new ValidationFailure(errors);
    }
    var sections = node.component1();
    var sectionMap;
    try {
      sectionMap = identifySections(sections, [section1Name, section2Name]);
    }
     catch (e) {
      if (Kotlin.isType(e, ParseError)) {
        errors.add_11rb$(new ParseError(e.message, e.row, e.column));
        return new ValidationFailure(errors);
      }
       else
        throw e;
    }
    var section1 = null;
    var sect1 = sectionMap.get_11rb$(section1Name);
    var section1Validation = validateSection1(ensureNotNull(sect1));
    if (Kotlin.isType(section1Validation, ValidationSuccess))
      section1 = section1Validation.value;
    else if (Kotlin.isType(section1Validation, ValidationFailure))
      errors.addAll_brywnq$(section1Validation.errors);
    else
      Kotlin.noWhenBranchMatched();
    var section2 = null;
    var sect2 = sectionMap.get_11rb$(section2Name);
    var section2Validation = validateSection2(ensureNotNull(sect2));
    if (Kotlin.isType(section2Validation, ValidationSuccess))
      section2 = section2Validation.value;
    else if (Kotlin.isType(section2Validation, ValidationFailure))
      errors.addAll_brywnq$(section2Validation.errors);
    else
      Kotlin.noWhenBranchMatched();
    if (!errors.isEmpty()) {
      tmp$ = new ValidationFailure(errors);
    }
     else
      tmp$ = new ValidationSuccess(buildGroup(ensureNotNull(section1), ensureNotNull(section2)));
    return tmp$;
  }
  function validateWrappedNode(rawNode, expectedType, checkType, build) {
    var node = rawNode.resolve();
    var base = checkType(node);
    if (base == null) {
      return new ValidationFailure(listOf(new ParseError('Cannot convert to a ' + expectedType, getRow(node), getColumn(node))));
    }
    return new ValidationSuccess(build(base));
  }
  function toCode(isArg, indent, phase1Node) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, ''));
    builder.append_gw00v9$(phase1Node.toCode());
    return builder.toString();
  }
  function toCode_0(isArg, indent, sections) {
    var builder = StringBuilder_init();
    for (var i = 0; i !== sections.length; ++i) {
      var sect = sections[i];
      if (sect != null) {
        builder.append_gw00v9$(sect.toCode_eltk6l$(isArg && i === 0, indent));
        if (i !== (sections.length - 1 | 0)) {
          builder.append_s8itvh$(10);
        }
      }
    }
    return builder.toString();
  }
  function ClauseListNode(clauses) {
    this.clauses = clauses;
  }
  ClauseListNode.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.clauses.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  ClauseListNode.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var tmp$;
    var builder = StringBuilder_init();
    tmp$ = this.clauses;
    for (var i = 0; i !== tmp$.size; ++i) {
      builder.append_gw00v9$(this.clauses.get_za3lpa$(i).toCode_eltk6l$(true, indent));
      if (i !== (this.clauses.size - 1 | 0)) {
        builder.append_s8itvh$(10);
      }
    }
    return builder.toString();
  };
  ClauseListNode.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var $receiver = this.clauses;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      var tmp$_0;
      destination.add_11rb$(Kotlin.isType(tmp$_0 = item.transform_nrl0ww$(chalkTransformer), Clause) ? tmp$_0 : throwCCE());
    }
    return chalkTransformer(new ClauseListNode(destination));
  };
  ClauseListNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ClauseListNode',
    interfaces: [Phase2Node]
  };
  ClauseListNode.prototype.component1 = function () {
    return this.clauses;
  };
  ClauseListNode.prototype.copy_9lvukv$ = function (clauses) {
    return new ClauseListNode(clauses === void 0 ? this.clauses : clauses);
  };
  ClauseListNode.prototype.toString = function () {
    return 'ClauseListNode(clauses=' + Kotlin.toString(this.clauses) + ')';
  };
  ClauseListNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.clauses) | 0;
    return result;
  };
  ClauseListNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.clauses, other.clauses))));
  };
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
  function validateClauseList(rawNode, expectedName, builder) {
    var tmp$;
    var node = rawNode.resolve();
    var validation = validate(node, expectedName);
    if (Kotlin.isType(validation, ValidationSuccess))
      tmp$ = new ValidationSuccess(builder(new ClauseListNode(validation.value.clauses)));
    else if (Kotlin.isType(validation, ValidationFailure))
      tmp$ = new ValidationFailure(validation.errors);
    else
      tmp$ = Kotlin.noWhenBranchMatched();
    return tmp$;
  }
  function validate(node, expectedName) {
    var tmp$, tmp$_0, tmp$_1;
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Section)) {
      errors.add_11rb$(new ParseError('Expected a Section', getRow(node), getColumn(node)));
    }
    var tmp$_2 = Kotlin.isType(tmp$ = node, Section) ? tmp$ : throwCCE();
    var name = tmp$_2.component1()
    , args = tmp$_2.component2();
    if (!equals(name.text, expectedName)) {
      errors.add_11rb$(new ParseError('Expected a Section with name ' + expectedName + ' but found ' + name.text, getRow(node), getColumn(node)));
    }
    if (args.isEmpty()) {
      errors.add_11rb$(new ParseError("Section '" + name.text + "' requires at least one argument.", getRow(node), getColumn(node)));
    }
    var clauses = ArrayList_init();
    tmp$_0 = args.iterator();
    while (tmp$_0.hasNext()) {
      var arg = tmp$_0.next();
      var validation = validateClause(arg);
      if (Kotlin.isType(validation, ValidationSuccess))
        clauses.add_11rb$(validation.value);
      else if (Kotlin.isType(validation, ValidationFailure))
        errors.addAll_brywnq$(validation.errors);
      else
        Kotlin.noWhenBranchMatched();
    }
    if (!errors.isEmpty()) {
      tmp$_1 = new ValidationFailure(errors);
    }
     else
      tmp$_1 = new ValidationSuccess(new ClauseListSection(name.text, clauses));
    return tmp$_1;
  }
  function Phase2Node() {
  }
  Phase2Node.$metadata$ = {
    kind: Kind_INTERFACE,
    simpleName: 'Phase2Node',
    interfaces: []
  };
  function Document(defines, represents, results, axioms, conjectures, sources) {
    this.defines = defines;
    this.represents = represents;
    this.results = results;
    this.axioms = axioms;
    this.conjectures = conjectures;
    this.sources = sources;
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
    var tmp$_4;
    tmp$_4 = this.sources.iterator();
    while (tmp$_4.hasNext()) {
      var element_4 = tmp$_4.next();
      fn(element_4);
    }
  };
  Document.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var tmp$, tmp$_0, tmp$_1, tmp$_2, tmp$_3, tmp$_4;
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
    tmp$_4 = this.sources.iterator();
    while (tmp$_4.hasNext()) {
      var src = tmp$_4.next();
      builder.append_gw00v9$(src.toCode_eltk6l$(false, 0));
      builder.append_gw00v9$('\n\n\n');
    }
    return builder.toString();
  };
  Document.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var $receiver = this.defines;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      var tmp$_0;
      destination.add_11rb$(Kotlin.isType(tmp$_0 = item.transform_nrl0ww$(chalkTransformer), DefinesGroup) ? tmp$_0 : throwCCE());
    }
    var $receiver_0 = this.axioms;
    var destination_0 = ArrayList_init_0(collectionSizeOrDefault($receiver_0, 10));
    var tmp$_1;
    tmp$_1 = $receiver_0.iterator();
    while (tmp$_1.hasNext()) {
      var item_0 = tmp$_1.next();
      var tmp$_2;
      destination_0.add_11rb$(Kotlin.isType(tmp$_2 = item_0.transform_nrl0ww$(chalkTransformer), AxiomGroup) ? tmp$_2 : throwCCE());
    }
    var $receiver_1 = this.conjectures;
    var destination_1 = ArrayList_init_0(collectionSizeOrDefault($receiver_1, 10));
    var tmp$_3;
    tmp$_3 = $receiver_1.iterator();
    while (tmp$_3.hasNext()) {
      var item_1 = tmp$_3.next();
      var tmp$_4;
      destination_1.add_11rb$(Kotlin.isType(tmp$_4 = item_1.transform_nrl0ww$(chalkTransformer), ConjectureGroup) ? tmp$_4 : throwCCE());
    }
    var $receiver_2 = this.represents;
    var destination_2 = ArrayList_init_0(collectionSizeOrDefault($receiver_2, 10));
    var tmp$_5;
    tmp$_5 = $receiver_2.iterator();
    while (tmp$_5.hasNext()) {
      var item_2 = tmp$_5.next();
      var tmp$_6;
      destination_2.add_11rb$(Kotlin.isType(tmp$_6 = item_2.transform_nrl0ww$(chalkTransformer), RepresentsGroup) ? tmp$_6 : throwCCE());
    }
    var $receiver_3 = this.results;
    var destination_3 = ArrayList_init_0(collectionSizeOrDefault($receiver_3, 10));
    var tmp$_7;
    tmp$_7 = $receiver_3.iterator();
    while (tmp$_7.hasNext()) {
      var item_3 = tmp$_7.next();
      var tmp$_8;
      destination_3.add_11rb$(Kotlin.isType(tmp$_8 = item_3.transform_nrl0ww$(chalkTransformer), ResultGroup) ? tmp$_8 : throwCCE());
    }
    var $receiver_4 = this.sources;
    var destination_4 = ArrayList_init_0(collectionSizeOrDefault($receiver_4, 10));
    var tmp$_9;
    tmp$_9 = $receiver_4.iterator();
    while (tmp$_9.hasNext()) {
      var item_4 = tmp$_9.next();
      var tmp$_10;
      destination_4.add_11rb$(Kotlin.isType(tmp$_10 = item_4.transform_nrl0ww$(chalkTransformer), SourceGroup) ? tmp$_10 : throwCCE());
    }
    return new Document(destination, destination_2, destination_3, destination_0, destination_1, destination_4);
  };
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
  Document.prototype.component6 = function () {
    return this.sources;
  };
  Document.prototype.copy_91x5o1$ = function (defines, represents, results, axioms, conjectures, sources) {
    return new Document(defines === void 0 ? this.defines : defines, represents === void 0 ? this.represents : represents, results === void 0 ? this.results : results, axioms === void 0 ? this.axioms : axioms, conjectures === void 0 ? this.conjectures : conjectures, sources === void 0 ? this.sources : sources);
  };
  Document.prototype.toString = function () {
    return 'Document(defines=' + Kotlin.toString(this.defines) + (', represents=' + Kotlin.toString(this.represents)) + (', results=' + Kotlin.toString(this.results)) + (', axioms=' + Kotlin.toString(this.axioms)) + (', conjectures=' + Kotlin.toString(this.conjectures)) + (', sources=' + Kotlin.toString(this.sources)) + ')';
  };
  Document.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.defines) | 0;
    result = result * 31 + Kotlin.hashCode(this.represents) | 0;
    result = result * 31 + Kotlin.hashCode(this.results) | 0;
    result = result * 31 + Kotlin.hashCode(this.axioms) | 0;
    result = result * 31 + Kotlin.hashCode(this.conjectures) | 0;
    result = result * 31 + Kotlin.hashCode(this.sources) | 0;
    return result;
  };
  Document.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.defines, other.defines) && Kotlin.equals(this.represents, other.represents) && Kotlin.equals(this.results, other.results) && Kotlin.equals(this.axioms, other.axioms) && Kotlin.equals(this.conjectures, other.conjectures) && Kotlin.equals(this.sources, other.sources)))));
  };
  function validateDocument(rawNode) {
    var tmp$, tmp$_0;
    var node = rawNode.resolve();
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Root)) {
      errors.add_11rb$(new ParseError('Expected a Root', getRow(node), getColumn(node)));
      return new ValidationFailure(errors);
    }
    var defines = ArrayList_init();
    var represents = ArrayList_init();
    var results = ArrayList_init();
    var axioms = ArrayList_init();
    var conjectures = ArrayList_init();
    var sources = ArrayList_init();
    var groups = node.component1();
    tmp$ = groups.iterator();
    while (tmp$.hasNext()) {
      var group = tmp$.next();
      if (isResultGroup(group)) {
        var resultValidation = validateResultGroup(group);
        if (Kotlin.isType(resultValidation, ValidationSuccess))
          results.add_11rb$(resultValidation.value);
        else if (Kotlin.isType(resultValidation, ValidationFailure))
          errors.addAll_brywnq$(resultValidation.errors);
        else
          Kotlin.noWhenBranchMatched();
      }
       else if (isAxiomGroup(group)) {
        var axiomValidation = validateAxiomGroup(group);
        if (Kotlin.isType(axiomValidation, ValidationSuccess))
          axioms.add_11rb$(axiomValidation.value);
        else if (Kotlin.isType(axiomValidation, ValidationFailure))
          errors.addAll_brywnq$(axiomValidation.errors);
        else
          Kotlin.noWhenBranchMatched();
      }
       else if (isConjectureGroup(group)) {
        var conjectureValidation = validateConjectureGroup(group);
        if (Kotlin.isType(conjectureValidation, ValidationSuccess))
          conjectures.add_11rb$(conjectureValidation.value);
        else if (Kotlin.isType(conjectureValidation, ValidationFailure))
          errors.addAll_brywnq$(conjectureValidation.errors);
        else
          Kotlin.noWhenBranchMatched();
      }
       else if (isDefinesGroup(group)) {
        var definesValidation = validateDefinesGroup(group);
        if (Kotlin.isType(definesValidation, ValidationSuccess))
          defines.add_11rb$(definesValidation.value);
        else if (Kotlin.isType(definesValidation, ValidationFailure))
          errors.addAll_brywnq$(definesValidation.errors);
        else
          Kotlin.noWhenBranchMatched();
      }
       else if (isRepresentsGroup(group)) {
        var representsValidation = validateRepresentsGroup(group);
        if (Kotlin.isType(representsValidation, ValidationSuccess))
          represents.add_11rb$(representsValidation.value);
        else if (Kotlin.isType(representsValidation, ValidationFailure))
          errors.addAll_brywnq$(representsValidation.errors);
        else
          Kotlin.noWhenBranchMatched();
      }
       else if (isSourceGroup(group)) {
        var sourceValidation = validateSourceGroup(group);
        if (Kotlin.isType(sourceValidation, ValidationSuccess))
          sources.add_11rb$(sourceValidation.value);
        else if (Kotlin.isType(sourceValidation, ValidationFailure))
          errors.addAll_brywnq$(sourceValidation.errors);
        else
          Kotlin.noWhenBranchMatched();
      }
       else {
        errors.add_11rb$(new ParseError('Expected a top level group but found ' + group.toCode(), getRow(group), getColumn(group)));
      }
    }
    if (!errors.isEmpty()) {
      tmp$_0 = new ValidationFailure(errors);
    }
     else
      tmp$_0 = new ValidationSuccess(new Document(defines, represents, results, axioms, conjectures, sources));
    return tmp$_0;
  }
  function SourceGroup(id, sourceSection) {
    this.id = id;
    this.sourceSection = sourceSection;
  }
  SourceGroup.prototype.forEach_ye21ev$ = function (fn) {
    fn(this.sourceSection);
  };
  SourceGroup.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return toCode_1(isArg, indent, new Statement(this.id, new ValidationFailure(emptyList())), [this.sourceSection]);
  };
  SourceGroup.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new SourceGroup(this.id, Kotlin.isType(tmp$ = this.sourceSection.transform_nrl0ww$(chalkTransformer), SourceSection) ? tmp$ : throwCCE()));
  };
  SourceGroup.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'SourceGroup',
    interfaces: [Phase2Node]
  };
  SourceGroup.prototype.component1 = function () {
    return this.id;
  };
  SourceGroup.prototype.component2 = function () {
    return this.sourceSection;
  };
  SourceGroup.prototype.copy_eesg3d$ = function (id, sourceSection) {
    return new SourceGroup(id === void 0 ? this.id : id, sourceSection === void 0 ? this.sourceSection : sourceSection);
  };
  SourceGroup.prototype.toString = function () {
    return 'SourceGroup(id=' + Kotlin.toString(this.id) + (', sourceSection=' + Kotlin.toString(this.sourceSection)) + ')';
  };
  SourceGroup.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.id) | 0;
    result = result * 31 + Kotlin.hashCode(this.sourceSection) | 0;
    return result;
  };
  SourceGroup.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.id, other.id) && Kotlin.equals(this.sourceSection, other.sourceSection)))));
  };
  function isSourceGroup(node) {
    return firstSectionMatchesName(node, 'Source');
  }
  function validateSourceGroup(groupNode) {
    var tmp$;
    var id = groupNode.id;
    if (id == null) {
      return new ValidationFailure(listOf(new ParseError('A Source group must have an id', getRow(groupNode), getColumn(groupNode))));
    }
    var $receiver = id.text;
    var endIndex = id.text.length - 1 | 0;
    var idText = $receiver.substring(1, endIndex);
    var errors = ArrayList_init();
    if (!Regex_init('[a-zA-Z0-9]+').matches_6bul2c$(idText)) {
      errors.add_11rb$(new ParseError('A source id can only contain numbers and letters', getRow(groupNode), getColumn(groupNode)));
    }
    var sections = groupNode.sections;
    if (sections.size !== 1) {
      errors.add_11rb$(new ParseError('Expected a singe section but found ' + sections.size, getRow(groupNode), getColumn(groupNode)));
    }
    var section = sections.get_za3lpa$(0);
    var validation = validateSourceSection(section);
    if (Kotlin.isType(validation, ValidationFailure)) {
      errors.addAll_brywnq$(validation.errors);
    }
    if (!errors.isEmpty()) {
      return new ValidationFailure(errors);
    }
    return new ValidationSuccess(new SourceGroup(idText, (Kotlin.isType(tmp$ = validation, ValidationSuccess) ? tmp$ : throwCCE()).value));
  }
  function DefinesGroup(signature, id, definesSection, assumingSection, meansSection, aliasSection, metaDataSection) {
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
  DefinesGroup.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$, tmp$_0, tmp$_1, tmp$_2, tmp$_3, tmp$_4, tmp$_5, tmp$_6, tmp$_7;
    return chalkTransformer(new DefinesGroup(this.signature, Kotlin.isType(tmp$ = this.id.transform_nrl0ww$(chalkTransformer), Statement) ? tmp$ : throwCCE(), Kotlin.isType(tmp$_0 = this.definesSection.transform_nrl0ww$(chalkTransformer), DefinesSection) ? tmp$_0 : throwCCE(), (tmp$_2 = (tmp$_1 = this.assumingSection) != null ? tmp$_1.transform_nrl0ww$(chalkTransformer) : null) == null || Kotlin.isType(tmp$_2, AssumingSection) ? tmp$_2 : throwCCE(), Kotlin.isType(tmp$_3 = this.meansSection.transform_nrl0ww$(chalkTransformer), MeansSection) ? tmp$_3 : throwCCE(), (tmp$_5 = (tmp$_4 = this.aliasSection) != null ? tmp$_4.transform_nrl0ww$(chalkTransformer) : null) == null || Kotlin.isType(tmp$_5, AliasSection) ? tmp$_5 : throwCCE(), (tmp$_7 = (tmp$_6 = this.metaDataSection) != null ? tmp$_6.transform_nrl0ww$(chalkTransformer) : null) == null || Kotlin.isType(tmp$_7, MetaDataSection) ? tmp$_7 : throwCCE()));
  };
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
  function isDefinesGroup(node) {
    return firstSectionMatchesName(node, 'Defines');
  }
  function validateDefinesGroup(groupNode) {
    return validateDefinesLikeGroup(groupNode, 'Defines', getCallableRef('validateDefinesSection', function (node) {
      return validateDefinesSection(node);
    }), 'means', getCallableRef('validateMeansSection', function (node) {
      return validateMeansSection(node);
    }), getCallableRef('DefinesGroup', function (signature, id, definesSection, assumingSection, meansSection, aliasSection, metaDataSection) {
      return new DefinesGroup(signature, id, definesSection, assumingSection, meansSection, aliasSection, metaDataSection);
    }));
  }
  function RepresentsGroup(signature, id, representsSection, assumingSection, thatSection, aliasSection, metaDataSection) {
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
  RepresentsGroup.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$, tmp$_0, tmp$_1, tmp$_2, tmp$_3, tmp$_4, tmp$_5, tmp$_6, tmp$_7;
    return chalkTransformer(new RepresentsGroup(this.signature, Kotlin.isType(tmp$ = this.id.transform_nrl0ww$(chalkTransformer), Statement) ? tmp$ : throwCCE(), Kotlin.isType(tmp$_0 = this.representsSection.transform_nrl0ww$(chalkTransformer), RepresentsSection) ? tmp$_0 : throwCCE(), (tmp$_2 = (tmp$_1 = this.assumingSection) != null ? tmp$_1.transform_nrl0ww$(chalkTransformer) : null) == null || Kotlin.isType(tmp$_2, AssumingSection) ? tmp$_2 : throwCCE(), Kotlin.isType(tmp$_3 = this.thatSection.transform_nrl0ww$(chalkTransformer), ThatSection) ? tmp$_3 : throwCCE(), (tmp$_5 = (tmp$_4 = this.aliasSection) != null ? tmp$_4.transform_nrl0ww$(chalkTransformer) : null) == null || Kotlin.isType(tmp$_5, AliasSection) ? tmp$_5 : throwCCE(), (tmp$_7 = (tmp$_6 = this.metaDataSection) != null ? tmp$_6.transform_nrl0ww$(chalkTransformer) : null) == null || Kotlin.isType(tmp$_7, MetaDataSection) ? tmp$_7 : throwCCE()));
  };
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
  function isRepresentsGroup(node) {
    return firstSectionMatchesName(node, 'Represents');
  }
  function validateRepresentsGroup(groupNode) {
    return validateDefinesLikeGroup(groupNode, 'Represents', getCallableRef('validateRepresentsSection', function (node) {
      return validateRepresentsSection(node);
    }), 'that', getCallableRef('validateThatSection', function (node) {
      return validateThatSection(node);
    }), getCallableRef('RepresentsGroup', function (signature, id, representsSection, assumingSection, thatSection, aliasSection, metaDataSection) {
      return new RepresentsGroup(signature, id, representsSection, assumingSection, thatSection, aliasSection, metaDataSection);
    }));
  }
  function ResultGroup(resultSection, aliasSection, metaDataSection) {
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
  ResultGroup.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$, tmp$_0, tmp$_1, tmp$_2, tmp$_3, tmp$_4, tmp$_5;
    tmp$_0 = Kotlin.isType(tmp$ = this.resultSection.transform_nrl0ww$(chalkTransformer), ResultSection) ? tmp$ : throwCCE();
    tmp$_3 = (tmp$_2 = (tmp$_1 = this.metaDataSection) != null ? tmp$_1.transform_nrl0ww$(chalkTransformer) : null) == null || Kotlin.isType(tmp$_2, MetaDataSection) ? tmp$_2 : throwCCE();
    return chalkTransformer(new ResultGroup(tmp$_0, (tmp$_5 = (tmp$_4 = this.aliasSection) != null ? tmp$_4.transform_nrl0ww$(chalkTransformer) : null) == null || Kotlin.isType(tmp$_5, AliasSection) ? tmp$_5 : throwCCE(), tmp$_3));
  };
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
  function isResultGroup(node) {
    return firstSectionMatchesName(node, 'Result');
  }
  function validateResultGroup(groupNode) {
    return validateResultLikeGroup(groupNode, 'Result', getCallableRef('validateResultSection', function (node) {
      return validateResultSection(node);
    }), getCallableRef('ResultGroup', function (resultSection, aliasSection, metaDataSection) {
      return new ResultGroup(resultSection, aliasSection, metaDataSection);
    }));
  }
  function AxiomGroup(axiomSection, aliasSection, metaDataSection) {
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
  AxiomGroup.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$, tmp$_0, tmp$_1, tmp$_2, tmp$_3;
    return chalkTransformer(new AxiomGroup(Kotlin.isType(tmp$ = this.axiomSection.transform_nrl0ww$(chalkTransformer), AxiomSection) ? tmp$ : throwCCE(), Kotlin.isType(tmp$_1 = (tmp$_0 = this.aliasSection) != null ? tmp$_0.transform_nrl0ww$(chalkTransformer) : null, AliasSection) ? tmp$_1 : throwCCE(), Kotlin.isType(tmp$_3 = (tmp$_2 = this.metaDataSection) != null ? tmp$_2.transform_nrl0ww$(chalkTransformer) : null, MetaDataSection) ? tmp$_3 : throwCCE()));
  };
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
  function isAxiomGroup(node) {
    return firstSectionMatchesName(node, 'Axiom');
  }
  function validateAxiomGroup(groupNode) {
    return validateResultLikeGroup(groupNode, 'Axiom', getCallableRef('validateAxiomSection', function (node) {
      return validateAxiomSection(node);
    }), getCallableRef('AxiomGroup', function (axiomSection, aliasSection, metaDataSection) {
      return new AxiomGroup(axiomSection, aliasSection, metaDataSection);
    }));
  }
  function ConjectureGroup(conjectureSection, aliasSection, metaDataSection) {
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
  ConjectureGroup.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$, tmp$_0, tmp$_1, tmp$_2, tmp$_3;
    return chalkTransformer(new ConjectureGroup(Kotlin.isType(tmp$ = this.conjectureSection.transform_nrl0ww$(chalkTransformer), ConjectureSection) ? tmp$ : throwCCE(), Kotlin.isType(tmp$_1 = (tmp$_0 = this.aliasSection) != null ? tmp$_0.transform_nrl0ww$(chalkTransformer) : null, AliasSection) ? tmp$_1 : throwCCE(), Kotlin.isType(tmp$_3 = (tmp$_2 = this.metaDataSection) != null ? tmp$_2.transform_nrl0ww$(chalkTransformer) : null, MetaDataSection) ? tmp$_3 : throwCCE()));
  };
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
  function isConjectureGroup(node) {
    return firstSectionMatchesName(node, 'Conjecture');
  }
  function validateConjectureGroup(groupNode) {
    return validateResultLikeGroup(groupNode, 'Conjecture', getCallableRef('validateConjectureSection', function (node) {
      return validateConjectureSection(node);
    }), getCallableRef('ConjectureGroup', function (conjectureSection, aliasSection, metaDataSection) {
      return new ConjectureGroup(conjectureSection, aliasSection, metaDataSection);
    }));
  }
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
      errors.add_11rb$(new ParseError('A result, axiom, or conjecture cannot have an Id', getRow(group), getColumn(group)));
    }
    var sections = group.sections;
    var sectionMap;
    try {
      sectionMap = identifySections(sections, [resultLikeName, 'Alias?', 'Metadata?']);
    }
     catch (e) {
      if (Kotlin.isType(e, ParseError)) {
        errors.add_11rb$(new ParseError(e.message, e.row, e.column));
        return new ValidationFailure(errors);
      }
       else
        throw e;
    }
    var resultLike = sectionMap.get_11rb$(resultLikeName);
    var alias = getOrNull(sectionMap, 'Alias');
    var metadata = getOrNull(sectionMap, 'Metadata');
    var resultLikeSection = null;
    var resultLikeValidation = validateResultLikeSection(ensureNotNull(resultLike));
    if (Kotlin.isType(resultLikeValidation, ValidationSuccess))
      resultLikeSection = resultLikeValidation.value;
    else if (Kotlin.isType(resultLikeValidation, ValidationFailure))
      errors.addAll_brywnq$(resultLikeValidation.errors);
    else
      Kotlin.noWhenBranchMatched();
    var metaDataSection = null;
    if (metadata != null) {
      var metaDataValidation = validateMetaDataSection(metadata);
      if (Kotlin.isType(metaDataValidation, ValidationSuccess))
        metaDataSection = metaDataValidation.value;
      else if (Kotlin.isType(metaDataValidation, ValidationFailure))
        errors.addAll_brywnq$(metaDataValidation.errors);
      else
        Kotlin.noWhenBranchMatched();
    }
    var aliasSection = null;
    if (alias != null) {
      var aliasValidation = validateAliasSection(alias);
      if (Kotlin.isType(aliasValidation, ValidationSuccess))
        aliasSection = aliasValidation.value;
      else if (Kotlin.isType(aliasValidation, ValidationFailure))
        errors.addAll_brywnq$(aliasValidation.errors);
      else
        Kotlin.noWhenBranchMatched();
    }
    if (!errors.isEmpty()) {
      tmp$_0 = new ValidationFailure(errors);
    }
     else
      tmp$_0 = new ValidationSuccess(buildGroup(ensureNotNull(resultLikeSection), aliasSection, metaDataSection));
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
      var stmtToken = new Phase1Token(statementText, ChalkTalkTokenType$Statement_getInstance(), row, column);
      var idValidation = validateStatement(stmtToken);
      if (Kotlin.isType(idValidation, ValidationSuccess))
        id = idValidation.value;
      else if (Kotlin.isType(idValidation, ValidationFailure))
        errors.addAll_brywnq$(idValidation.errors);
      else
        Kotlin.noWhenBranchMatched();
    }
     else {
      errors.add_11rb$(new ParseError('A definition must have an Id', getRow(group), getColumn(group)));
    }
    var sections = group.sections;
    var sectionMap;
    try {
      sectionMap = identifySections(sections, [definesLikeSectionName, 'assuming?', endSectionName, 'Alias?', 'Metadata?']);
    }
     catch (e) {
      if (Kotlin.isType(e, ParseError)) {
        errors.add_11rb$(new ParseError(e.message, e.row, e.column));
        return new ValidationFailure(errors);
      }
       else
        throw e;
    }
    var definesLike = sectionMap.get_11rb$(definesLikeSectionName);
    var assuming = getOrNull(sectionMap, 'assuming');
    var end = sectionMap.get_11rb$(endSectionName);
    var alias = getOrNull(sectionMap, 'Alias');
    var metadata = getOrNull(sectionMap, 'Metadata');
    var definesLikeSection = null;
    var definesLikeValidation = validateDefinesLikeSection(ensureNotNull(definesLike));
    if (Kotlin.isType(definesLikeValidation, ValidationSuccess))
      definesLikeSection = definesLikeValidation.value;
    else if (Kotlin.isType(definesLikeValidation, ValidationFailure))
      errors.addAll_brywnq$(definesLikeValidation.errors);
    else
      Kotlin.noWhenBranchMatched();
    var assumingSection = null;
    if (assuming != null) {
      var assumingValidation = validateAssumingSection(assuming);
      if (Kotlin.isType(assumingValidation, ValidationSuccess))
        assumingSection = assumingValidation.value;
      else if (Kotlin.isType(assumingValidation, ValidationFailure))
        errors.addAll_brywnq$(assumingValidation.errors);
      else
        Kotlin.noWhenBranchMatched();
    }
    var endSection = null;
    var endValidation = validateEndSection(ensureNotNull(end));
    if (Kotlin.isType(endValidation, ValidationSuccess))
      endSection = endValidation.value;
    else if (Kotlin.isType(endValidation, ValidationFailure))
      errors.addAll_brywnq$(endValidation.errors);
    else
      Kotlin.noWhenBranchMatched();
    var aliasSection = null;
    if (alias != null) {
      var aliasValidation = validateAliasSection(alias);
      if (Kotlin.isType(aliasValidation, ValidationSuccess))
        aliasSection = aliasValidation.value;
      else if (Kotlin.isType(aliasValidation, ValidationFailure))
        errors.addAll_brywnq$(aliasValidation.errors);
      else
        Kotlin.noWhenBranchMatched();
    }
    var metaDataSection = null;
    if (metadata != null) {
      var metaDataValidation = validateMetaDataSection(metadata);
      if (Kotlin.isType(metaDataValidation, ValidationSuccess))
        metaDataSection = metaDataValidation.value;
      else if (Kotlin.isType(metaDataValidation, ValidationFailure))
        errors.addAll_brywnq$(metaDataValidation.errors);
      else
        Kotlin.noWhenBranchMatched();
    }
    if (!errors.isEmpty()) {
      tmp$_0 = new ValidationFailure(errors);
    }
     else
      tmp$_0 = new ValidationSuccess(buildGroup(getSignature(ensureNotNull(id)), id, ensureNotNull(definesLikeSection), assumingSection, ensureNotNull(endSection), aliasSection, metaDataSection));
    return tmp$_0;
  }
  function getOrNull($receiver, key) {
    return $receiver.containsKey_11rb$(key) ? $receiver.get_11rb$(key) : null;
  }
  function MetaDataSection(mappings) {
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
    tmp$ = this.mappings;
    for (var i = 0; i !== tmp$.size; ++i) {
      builder.append_gw00v9$(this.mappings.get_za3lpa$(i).toCode_eltk6l$(true, indent + 2 | 0));
      if (i !== (this.mappings.size - 1 | 0)) {
        builder.append_s8itvh$(10);
      }
    }
    return builder.toString();
  };
  MetaDataSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var $receiver = this.mappings;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      var tmp$_0;
      destination.add_11rb$(Kotlin.isType(tmp$_0 = item.transform_nrl0ww$(chalkTransformer), MappingNode) ? tmp$_0 : throwCCE());
    }
    return chalkTransformer(new MetaDataSection(destination));
  };
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
  function validateMetaDataSection(section) {
    var tmp$, tmp$_0;
    if (!equals(section.name.text, 'Metadata')) {
      return new ValidationFailure(listOf(new ParseError("Expected a 'Metadata' but found '" + section.name.text + "'", getRow(section), getColumn(section))));
    }
    var errors = ArrayList_init();
    var mappings = ArrayList_init();
    tmp$ = section.args.iterator();
    while (tmp$.hasNext()) {
      var arg = tmp$.next();
      var validation = validateMappingNode(arg);
      if (Kotlin.isType(validation, ValidationSuccess))
        mappings.add_11rb$(validation.value);
      else if (Kotlin.isType(validation, ValidationFailure))
        errors.addAll_brywnq$(validation.errors);
      else
        Kotlin.noWhenBranchMatched();
    }
    if (!errors.isEmpty()) {
      tmp$_0 = new ValidationFailure(errors);
    }
     else {
      tmp$_0 = new ValidationSuccess(new MetaDataSection(mappings));
    }
    return tmp$_0;
  }
  function appendTargetArgs(builder, targets, indent) {
    for (var i = 0; i !== targets.size; ++i) {
      builder.append_gw00v9$(targets.get_za3lpa$(i).toCode_eltk6l$(true, indent));
      if (i !== (targets.size - 1 | 0)) {
        builder.append_s8itvh$(10);
      }
    }
  }
  function AssumingSection(clauses) {
    this.clauses = clauses;
  }
  AssumingSection.prototype.forEach_ye21ev$ = function (fn) {
    this.clauses.forEach_ye21ev$(fn);
  };
  AssumingSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'assuming:'));
    if (!this.clauses.clauses.isEmpty()) {
      builder.append_s8itvh$(10);
    }
    builder.append_gw00v9$(this.clauses.toCode_eltk6l$(true, indent + 2 | 0));
    return builder.toString();
  };
  AssumingSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new AssumingSection(Kotlin.isType(tmp$ = this.clauses.transform_nrl0ww$(chalkTransformer), ClauseListNode) ? tmp$ : throwCCE()));
  };
  AssumingSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'AssumingSection',
    interfaces: [Phase2Node]
  };
  AssumingSection.prototype.component1 = function () {
    return this.clauses;
  };
  AssumingSection.prototype.copy_392qqo$ = function (clauses) {
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
  function validateAssumingSection$lambda(it) {
    return new AssumingSection(it);
  }
  function validateAssumingSection(node) {
    return validateClauseList(node, 'assuming', validateAssumingSection$lambda);
  }
  function DefinesSection(targets) {
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
  DefinesSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var $receiver = this.targets;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      var tmp$_0;
      destination.add_11rb$(Kotlin.isType(tmp$_0 = item.transform_nrl0ww$(chalkTransformer), Target) ? tmp$_0 : throwCCE());
    }
    return chalkTransformer(new DefinesSection(destination));
  };
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
  function validateDefinesSection$lambda(it) {
    return new DefinesSection(it);
  }
  function validateDefinesSection(node) {
    return validateTargetList(node, 'Defines', validateDefinesSection$lambda);
  }
  function RefinesSection(targets) {
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
  RefinesSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var $receiver = this.targets;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      var tmp$_0;
      destination.add_11rb$(Kotlin.isType(tmp$_0 = item.transform_nrl0ww$(chalkTransformer), Target) ? tmp$_0 : throwCCE());
    }
    return chalkTransformer(new RefinesSection(destination));
  };
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
  function validateRefinesSection$lambda(it) {
    return new RefinesSection(it);
  }
  function validateRefinesSection(node) {
    return validateTargetList(node, 'Refines', validateRefinesSection$lambda);
  }
  function RepresentsSection() {
  }
  RepresentsSection.prototype.forEach_ye21ev$ = function (fn) {
  };
  RepresentsSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    return indentedString(isArg, indent, 'Represents:');
  };
  RepresentsSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    return chalkTransformer(this);
  };
  RepresentsSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'RepresentsSection',
    interfaces: [Phase2Node]
  };
  function validateRepresentsSection(node) {
    var tmp$, tmp$_0;
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Section)) {
      errors.add_11rb$(new ParseError('Expected a RepresentsSection', getRow(node), getColumn(node)));
    }
    var sect = Kotlin.isType(tmp$ = node, Section) ? tmp$ : throwCCE();
    if (!sect.args.isEmpty()) {
      errors.add_11rb$(new ParseError('A Represents cannot have any arguments', getRow(node), getColumn(node)));
    }
    if (!equals(sect.name.text, 'Represents')) {
      errors.add_11rb$(new ParseError('Expected a section named Represents', getRow(node), getColumn(node)));
    }
    if (!errors.isEmpty()) {
      tmp$_0 = new ValidationFailure(errors);
    }
     else {
      tmp$_0 = new ValidationSuccess(new RepresentsSection());
    }
    return tmp$_0;
  }
  function ExistsSection(identifiers) {
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
  ExistsSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var $receiver = this.identifiers;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      var tmp$_0;
      destination.add_11rb$(Kotlin.isType(tmp$_0 = item.transform_nrl0ww$(chalkTransformer), Target) ? tmp$_0 : throwCCE());
    }
    return chalkTransformer(new ExistsSection(destination));
  };
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
  function validateExistsSection$lambda(it) {
    return new ExistsSection(it);
  }
  function validateExistsSection(node) {
    return validateTargetList(node, 'exists', validateExistsSection$lambda);
  }
  function ForSection(targets) {
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
  ForSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var $receiver = this.targets;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      var tmp$_0;
      destination.add_11rb$(Kotlin.isType(tmp$_0 = item.transform_nrl0ww$(chalkTransformer), Target) ? tmp$_0 : throwCCE());
    }
    return chalkTransformer(new ForSection(destination));
  };
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
  function validateForSection$lambda(it) {
    return new ForSection(it);
  }
  function validateForSection(node) {
    return validateTargetList(node, 'for', validateForSection$lambda);
  }
  function MeansSection(clauses) {
    this.clauses = clauses;
  }
  MeansSection.prototype.forEach_ye21ev$ = function (fn) {
    fn(this.clauses);
  };
  MeansSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'means:'));
    if (!this.clauses.clauses.isEmpty()) {
      builder.append_s8itvh$(10);
    }
    builder.append_gw00v9$(this.clauses.toCode_eltk6l$(true, indent + 2 | 0));
    return builder.toString();
  };
  MeansSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new MeansSection(Kotlin.isType(tmp$ = this.clauses.transform_nrl0ww$(chalkTransformer), ClauseListNode) ? tmp$ : throwCCE()));
  };
  MeansSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'MeansSection',
    interfaces: [Phase2Node]
  };
  MeansSection.prototype.component1 = function () {
    return this.clauses;
  };
  MeansSection.prototype.copy_392qqo$ = function (clauses) {
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
  function validateMeansSection$lambda(it) {
    return new MeansSection(it);
  }
  function validateMeansSection(node) {
    return validateClauseList(node, 'means', validateMeansSection$lambda);
  }
  function ResultSection(clauses) {
    this.clauses = clauses;
  }
  ResultSection.prototype.forEach_ye21ev$ = function (fn) {
    this.clauses.forEach_ye21ev$(fn);
  };
  ResultSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'Result:'));
    if (!this.clauses.clauses.isEmpty()) {
      builder.append_s8itvh$(10);
    }
    builder.append_gw00v9$(this.clauses.toCode_eltk6l$(true, indent + 2 | 0));
    return builder.toString();
  };
  ResultSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new ResultSection(Kotlin.isType(tmp$ = this.clauses.transform_nrl0ww$(chalkTransformer), ClauseListNode) ? tmp$ : throwCCE()));
  };
  ResultSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ResultSection',
    interfaces: [Phase2Node]
  };
  ResultSection.prototype.component1 = function () {
    return this.clauses;
  };
  ResultSection.prototype.copy_392qqo$ = function (clauses) {
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
  function validateResultSection$lambda(it) {
    return new ResultSection(it);
  }
  function validateResultSection(node) {
    return validateClauseList(node, 'Result', validateResultSection$lambda);
  }
  function AxiomSection(clauses) {
    this.clauses = clauses;
  }
  AxiomSection.prototype.forEach_ye21ev$ = function (fn) {
    this.clauses.forEach_ye21ev$(fn);
  };
  AxiomSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'Axiom:'));
    if (!this.clauses.clauses.isEmpty()) {
      builder.append_s8itvh$(10);
    }
    builder.append_gw00v9$(this.clauses.toCode_eltk6l$(true, indent + 2 | 0));
    return builder.toString();
  };
  AxiomSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new AxiomSection(Kotlin.isType(tmp$ = this.clauses.transform_nrl0ww$(chalkTransformer), ClauseListNode) ? tmp$ : throwCCE()));
  };
  AxiomSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'AxiomSection',
    interfaces: [Phase2Node]
  };
  AxiomSection.prototype.component1 = function () {
    return this.clauses;
  };
  AxiomSection.prototype.copy_392qqo$ = function (clauses) {
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
  function validateAxiomSection$lambda(it) {
    return new AxiomSection(it);
  }
  function validateAxiomSection(node) {
    return validateClauseList(node, 'Axiom', validateAxiomSection$lambda);
  }
  function ConjectureSection(clauses) {
    this.clauses = clauses;
  }
  ConjectureSection.prototype.forEach_ye21ev$ = function (fn) {
    this.clauses.forEach_ye21ev$(fn);
  };
  ConjectureSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'Conjecture:'));
    if (!this.clauses.clauses.isEmpty()) {
      builder.append_s8itvh$(10);
    }
    builder.append_gw00v9$(this.clauses.toCode_eltk6l$(true, indent + 2 | 0));
    return builder.toString();
  };
  ConjectureSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new ConjectureSection(Kotlin.isType(tmp$ = this.clauses.transform_nrl0ww$(chalkTransformer), ClauseListNode) ? tmp$ : throwCCE()));
  };
  ConjectureSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ConjectureSection',
    interfaces: [Phase2Node]
  };
  ConjectureSection.prototype.component1 = function () {
    return this.clauses;
  };
  ConjectureSection.prototype.copy_392qqo$ = function (clauses) {
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
  function validateConjectureSection$lambda(it) {
    return new ConjectureSection(it);
  }
  function validateConjectureSection(node) {
    return validateClauseList(node, 'Conjecture', validateConjectureSection$lambda);
  }
  function SuchThatSection(clauses) {
    this.clauses = clauses;
  }
  SuchThatSection.prototype.forEach_ye21ev$ = function (fn) {
    this.clauses.forEach_ye21ev$(fn);
  };
  SuchThatSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'suchThat:'));
    if (!this.clauses.clauses.isEmpty()) {
      builder.append_s8itvh$(10);
    }
    builder.append_gw00v9$(this.clauses.toCode_eltk6l$(true, indent + 2 | 0));
    return builder.toString();
  };
  SuchThatSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new SuchThatSection(Kotlin.isType(tmp$ = this.clauses.transform_nrl0ww$(chalkTransformer), ClauseListNode) ? tmp$ : throwCCE()));
  };
  SuchThatSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'SuchThatSection',
    interfaces: [Phase2Node]
  };
  SuchThatSection.prototype.component1 = function () {
    return this.clauses;
  };
  SuchThatSection.prototype.copy_392qqo$ = function (clauses) {
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
  function validateSuchThatSection$lambda(it) {
    return new SuchThatSection(it);
  }
  function validateSuchThatSection(node) {
    return validateClauseList(node, 'suchThat', validateSuchThatSection$lambda);
  }
  function ThatSection(clauses) {
    this.clauses = clauses;
  }
  ThatSection.prototype.forEach_ye21ev$ = function (fn) {
    this.clauses.forEach_ye21ev$(fn);
  };
  ThatSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'that:'));
    if (!this.clauses.clauses.isEmpty()) {
      builder.append_s8itvh$(10);
    }
    builder.append_gw00v9$(this.clauses.toCode_eltk6l$(true, indent + 2 | 0));
    return builder.toString();
  };
  ThatSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new ThatSection(Kotlin.isType(tmp$ = this.clauses.transform_nrl0ww$(chalkTransformer), ClauseListNode) ? tmp$ : throwCCE()));
  };
  ThatSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ThatSection',
    interfaces: [Phase2Node]
  };
  ThatSection.prototype.component1 = function () {
    return this.clauses;
  };
  ThatSection.prototype.copy_392qqo$ = function (clauses) {
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
  function validateThatSection$lambda(it) {
    return new ThatSection(it);
  }
  function validateThatSection(node) {
    return validateClauseList(node, 'that', validateThatSection$lambda);
  }
  function IfSection(clauses) {
    this.clauses = clauses;
  }
  IfSection.prototype.forEach_ye21ev$ = function (fn) {
    this.clauses.forEach_ye21ev$(fn);
  };
  IfSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'if:'));
    if (!this.clauses.clauses.isEmpty()) {
      builder.append_s8itvh$(10);
    }
    builder.append_gw00v9$(this.clauses.toCode_eltk6l$(true, indent + 2 | 0));
    return builder.toString();
  };
  IfSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new IfSection(Kotlin.isType(tmp$ = this.clauses.transform_nrl0ww$(chalkTransformer), ClauseListNode) ? tmp$ : throwCCE()));
  };
  IfSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'IfSection',
    interfaces: [Phase2Node]
  };
  IfSection.prototype.component1 = function () {
    return this.clauses;
  };
  IfSection.prototype.copy_392qqo$ = function (clauses) {
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
  function validateIfSection$lambda(it) {
    return new IfSection(it);
  }
  function validateIfSection(node) {
    return validateClauseList(node, 'if', validateIfSection$lambda);
  }
  function IffSection(clauses) {
    this.clauses = clauses;
  }
  IffSection.prototype.forEach_ye21ev$ = function (fn) {
    this.clauses.forEach_ye21ev$(fn);
  };
  IffSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'iff:'));
    if (!this.clauses.clauses.isEmpty()) {
      builder.append_s8itvh$(10);
    }
    builder.append_gw00v9$(this.clauses.toCode_eltk6l$(true, indent + 2 | 0));
    return builder.toString();
  };
  IffSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new IffSection(Kotlin.isType(tmp$ = this.clauses.transform_nrl0ww$(chalkTransformer), ClauseListNode) ? tmp$ : throwCCE()));
  };
  IffSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'IffSection',
    interfaces: [Phase2Node]
  };
  IffSection.prototype.component1 = function () {
    return this.clauses;
  };
  IffSection.prototype.copy_392qqo$ = function (clauses) {
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
  function validateIffSection$lambda(it) {
    return new IffSection(it);
  }
  function validateIffSection(node) {
    return validateClauseList(node, 'iff', validateIffSection$lambda);
  }
  function ThenSection(clauses) {
    this.clauses = clauses;
  }
  ThenSection.prototype.forEach_ye21ev$ = function (fn) {
    this.clauses.forEach_ye21ev$(fn);
  };
  ThenSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'then:'));
    if (!this.clauses.clauses.isEmpty()) {
      builder.append_s8itvh$(10);
    }
    builder.append_gw00v9$(this.clauses.toCode_eltk6l$(true, indent + 2 | 0));
    return builder.toString();
  };
  ThenSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new ThenSection(Kotlin.isType(tmp$ = this.clauses.transform_nrl0ww$(chalkTransformer), ClauseListNode) ? tmp$ : throwCCE()));
  };
  ThenSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ThenSection',
    interfaces: [Phase2Node]
  };
  ThenSection.prototype.component1 = function () {
    return this.clauses;
  };
  ThenSection.prototype.copy_392qqo$ = function (clauses) {
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
  function validateThenSection$lambda(it) {
    return new ThenSection(it);
  }
  function validateThenSection(node) {
    return validateClauseList(node, 'then', validateThenSection$lambda);
  }
  function WhereSection(clauses) {
    this.clauses = clauses;
  }
  WhereSection.prototype.forEach_ye21ev$ = function (fn) {
    this.clauses.forEach_ye21ev$(fn);
  };
  WhereSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'where:'));
    if (!this.clauses.clauses.isEmpty()) {
      builder.append_s8itvh$(10);
    }
    builder.append_gw00v9$(this.clauses.toCode_eltk6l$(true, indent + 2 | 0));
    return builder.toString();
  };
  WhereSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new WhereSection(Kotlin.isType(tmp$ = this.clauses.transform_nrl0ww$(chalkTransformer), ClauseListNode) ? tmp$ : throwCCE()));
  };
  WhereSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'WhereSection',
    interfaces: [Phase2Node]
  };
  WhereSection.prototype.component1 = function () {
    return this.clauses;
  };
  WhereSection.prototype.copy_392qqo$ = function (clauses) {
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
  function validateWhereSection$lambda(it) {
    return new WhereSection(it);
  }
  function validateWhereSection(node) {
    return validateClauseList(node, 'where', validateWhereSection$lambda);
  }
  function NotSection(clauses) {
    this.clauses = clauses;
  }
  NotSection.prototype.forEach_ye21ev$ = function (fn) {
    this.clauses.forEach_ye21ev$(fn);
  };
  NotSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'not:'));
    if (!this.clauses.clauses.isEmpty()) {
      builder.append_s8itvh$(10);
    }
    builder.append_gw00v9$(this.clauses.toCode_eltk6l$(true, indent + 2 | 0));
    return builder.toString();
  };
  NotSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new NotSection(Kotlin.isType(tmp$ = this.clauses.transform_nrl0ww$(chalkTransformer), ClauseListNode) ? tmp$ : throwCCE()));
  };
  NotSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'NotSection',
    interfaces: [Phase2Node]
  };
  NotSection.prototype.component1 = function () {
    return this.clauses;
  };
  NotSection.prototype.copy_392qqo$ = function (clauses) {
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
  function validateNotSection$lambda(it) {
    return new NotSection(it);
  }
  function validateNotSection(node) {
    return validateClauseList(node, 'not', validateNotSection$lambda);
  }
  function OrSection(clauses) {
    this.clauses = clauses;
  }
  OrSection.prototype.forEach_ye21ev$ = function (fn) {
    this.clauses.forEach_ye21ev$(fn);
  };
  OrSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'or:'));
    if (!this.clauses.clauses.isEmpty()) {
      builder.append_s8itvh$(10);
    }
    builder.append_gw00v9$(this.clauses.toCode_eltk6l$(true, indent + 2 | 0));
    return builder.toString();
  };
  OrSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    var tmp$;
    return chalkTransformer(new OrSection(Kotlin.isType(tmp$ = this.clauses.transform_nrl0ww$(chalkTransformer), ClauseListNode) ? tmp$ : throwCCE()));
  };
  OrSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'OrSection',
    interfaces: [Phase2Node]
  };
  OrSection.prototype.component1 = function () {
    return this.clauses;
  };
  OrSection.prototype.copy_392qqo$ = function (clauses) {
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
  function validateOrSection$lambda(it) {
    return new OrSection(it);
  }
  function validateOrSection(node) {
    return validateClauseList(node, 'or', validateOrSection$lambda);
  }
  function identifySections(sections, expected) {
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
        throw new ParseError('For pattern:\n\n' + pattern + "\nExpected '" + trueName + "' but found '" + nextSection.name.text + "'", getRow(nextSection), getColumn(nextSection));
      }
    }
    if (!sectionQueue.isEmpty()) {
      var peek = sectionQueue.peek();
      throw new ParseError('For pattern:\n\n' + pattern + "\nUnexpected Section '" + peek.name.text + "'", getRow(peek), getColumn(peek));
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
      startRow = getRow(sect);
      startColumn = getColumn(sect);
    }
    if (nextExpected != null) {
      throw new ParseError('For pattern:\n\n' + pattern + '\nExpected a ' + nextExpected, startRow, startColumn);
    }
    return result;
  }
  function SourceSection(mappings) {
    this.mappings = mappings;
  }
  SourceSection.prototype.forEach_ye21ev$ = function (fn) {
    var tmp$;
    tmp$ = this.mappings.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  SourceSection.prototype.toCode_eltk6l$ = function (isArg, indent) {
    var tmp$;
    var builder = StringBuilder_init();
    builder.append_gw00v9$(indentedString(isArg, indent, 'Source:'));
    builder.append_s8itvh$(10);
    tmp$ = this.mappings;
    for (var i = 0; i !== tmp$.size; ++i) {
      builder.append_gw00v9$(this.mappings.get_za3lpa$(i).toCode_eltk6l$(true, indent + 2 | 0));
      if (i !== (this.mappings.size - 1 | 0)) {
        builder.append_s8itvh$(10);
      }
    }
    return builder.toString();
  };
  SourceSection.prototype.transform_nrl0ww$ = function (chalkTransformer) {
    return chalkTransformer(this);
  };
  SourceSection.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'SourceSection',
    interfaces: [Phase2Node]
  };
  SourceSection.prototype.component1 = function () {
    return this.mappings;
  };
  SourceSection.prototype.copy_rz3npo$ = function (mappings) {
    return new SourceSection(mappings === void 0 ? this.mappings : mappings);
  };
  SourceSection.prototype.toString = function () {
    return 'SourceSection(mappings=' + Kotlin.toString(this.mappings) + ')';
  };
  SourceSection.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.mappings) | 0;
    return result;
  };
  SourceSection.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.mappings, other.mappings))));
  };
  function validateSourceSection(section) {
    var tmp$, tmp$_0;
    if (!equals(section.name.text, 'Source')) {
      return new ValidationFailure(listOf(new ParseError("Expected a 'Source' but found '" + section.name.text + "'", getRow(section), getColumn(section))));
    }
    var errors = ArrayList_init();
    var mappings = ArrayList_init();
    tmp$ = section.args.iterator();
    while (tmp$.hasNext()) {
      var arg = tmp$.next();
      var validation = validateMappingNode(arg);
      if (Kotlin.isType(validation, ValidationSuccess))
        mappings.add_11rb$(validation.value);
      else if (Kotlin.isType(validation, ValidationFailure))
        errors.addAll_brywnq$(validation.errors);
      else
        Kotlin.noWhenBranchMatched();
    }
    if (!errors.isEmpty()) {
      tmp$_0 = new ValidationFailure(errors);
    }
     else {
      tmp$_0 = new ValidationSuccess(new SourceSection(mappings));
    }
    return tmp$_0;
  }
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
  function validateTargetList(rawNode, expectedName, builder) {
    var tmp$;
    var node = rawNode.resolve();
    var validation = validate_0(node, expectedName);
    if (Kotlin.isType(validation, ValidationSuccess)) {
      var targets = validation.value.targets;
      return new ValidationSuccess(builder(targets));
    }
     else if (Kotlin.isType(validation, ValidationFailure))
      tmp$ = new ValidationFailure(validation.errors);
    else
      tmp$ = Kotlin.noWhenBranchMatched();
    return tmp$;
  }
  function validate_0(node, expectedName) {
    var tmp$, tmp$_0, tmp$_1;
    var errors = ArrayList_init();
    if (!Kotlin.isType(node, Section)) {
      errors.add_11rb$(new ParseError('Expected a Section', getRow(node), getColumn(node)));
    }
    var tmp$_2 = Kotlin.isType(tmp$ = node, Section) ? tmp$ : throwCCE();
    var name1 = tmp$_2.component1()
    , args = tmp$_2.component2();
    var name = name1.text;
    if (!equals(name, expectedName)) {
      errors.add_11rb$(new ParseError('Expected a Section with name ' + expectedName + ' but found ' + name, getRow(node), getColumn(node)));
    }
    var targets = ArrayList_init();
    if (args.isEmpty()) {
      errors.add_11rb$(new ParseError("Section '" + name1.text + "' requires at least one argument.", getRow(node), getColumn(node)));
    }
    tmp$_0 = args.iterator();
    while (tmp$_0.hasNext()) {
      var arg = tmp$_0.next();
      var shouldContinue = false;
      var clauseValidation = validateClause(arg);
      if (Kotlin.isType(clauseValidation, ValidationSuccess)) {
        var clause = clauseValidation.value;
        if (Kotlin.isType(clause, Target)) {
          targets.add_11rb$(clause);
          shouldContinue = true;
        }
      }
       else if (Kotlin.isType(clauseValidation, ValidationFailure))
        errors.addAll_brywnq$(clauseValidation.errors);
      else
        Kotlin.noWhenBranchMatched();
      if (shouldContinue) {
        continue;
      }
      errors.add_11rb$(new ParseError('Expected an Target', getRow(arg), getColumn(arg)));
    }
    if (!errors.isEmpty()) {
      tmp$_1 = new ValidationFailure(errors);
    }
     else
      tmp$_1 = new ValidationSuccess(new TargetListSection(targets));
    return tmp$_1;
  }
  function TexTalkNodeType(name, ordinal) {
    Enum.call(this);
    this.name$ = name;
    this.ordinal$ = ordinal;
  }
  function TexTalkNodeType_initFields() {
    TexTalkNodeType_initFields = function () {
    };
    TexTalkNodeType$Token_instance = new TexTalkNodeType('Token', 0);
    TexTalkNodeType$Identifier_instance = new TexTalkNodeType('Identifier', 1);
    TexTalkNodeType$Operator_instance = new TexTalkNodeType('Operator', 2);
    TexTalkNodeType$ParenGroup_instance = new TexTalkNodeType('ParenGroup', 3);
    TexTalkNodeType$SquareGroup_instance = new TexTalkNodeType('SquareGroup', 4);
    TexTalkNodeType$CurlyGroup_instance = new TexTalkNodeType('CurlyGroup', 5);
    TexTalkNodeType$NamedGroup_instance = new TexTalkNodeType('NamedGroup', 6);
    TexTalkNodeType$Command_instance = new TexTalkNodeType('Command', 7);
    TexTalkNodeType$CommandPart_instance = new TexTalkNodeType('CommandPart', 8);
    TexTalkNodeType$Expression_instance = new TexTalkNodeType('Expression', 9);
    TexTalkNodeType$SubSup_instance = new TexTalkNodeType('SubSup', 10);
    TexTalkNodeType$Parameters_instance = new TexTalkNodeType('Parameters', 11);
    TexTalkNodeType$Comma_instance = new TexTalkNodeType('Comma', 12);
    TexTalkNodeType$Is_instance = new TexTalkNodeType('Is', 13);
    TexTalkNodeType$ColonEquals_instance = new TexTalkNodeType('ColonEquals', 14);
  }
  var TexTalkNodeType$Token_instance;
  function TexTalkNodeType$Token_getInstance() {
    TexTalkNodeType_initFields();
    return TexTalkNodeType$Token_instance;
  }
  var TexTalkNodeType$Identifier_instance;
  function TexTalkNodeType$Identifier_getInstance() {
    TexTalkNodeType_initFields();
    return TexTalkNodeType$Identifier_instance;
  }
  var TexTalkNodeType$Operator_instance;
  function TexTalkNodeType$Operator_getInstance() {
    TexTalkNodeType_initFields();
    return TexTalkNodeType$Operator_instance;
  }
  var TexTalkNodeType$ParenGroup_instance;
  function TexTalkNodeType$ParenGroup_getInstance() {
    TexTalkNodeType_initFields();
    return TexTalkNodeType$ParenGroup_instance;
  }
  var TexTalkNodeType$SquareGroup_instance;
  function TexTalkNodeType$SquareGroup_getInstance() {
    TexTalkNodeType_initFields();
    return TexTalkNodeType$SquareGroup_instance;
  }
  var TexTalkNodeType$CurlyGroup_instance;
  function TexTalkNodeType$CurlyGroup_getInstance() {
    TexTalkNodeType_initFields();
    return TexTalkNodeType$CurlyGroup_instance;
  }
  var TexTalkNodeType$NamedGroup_instance;
  function TexTalkNodeType$NamedGroup_getInstance() {
    TexTalkNodeType_initFields();
    return TexTalkNodeType$NamedGroup_instance;
  }
  var TexTalkNodeType$Command_instance;
  function TexTalkNodeType$Command_getInstance() {
    TexTalkNodeType_initFields();
    return TexTalkNodeType$Command_instance;
  }
  var TexTalkNodeType$CommandPart_instance;
  function TexTalkNodeType$CommandPart_getInstance() {
    TexTalkNodeType_initFields();
    return TexTalkNodeType$CommandPart_instance;
  }
  var TexTalkNodeType$Expression_instance;
  function TexTalkNodeType$Expression_getInstance() {
    TexTalkNodeType_initFields();
    return TexTalkNodeType$Expression_instance;
  }
  var TexTalkNodeType$SubSup_instance;
  function TexTalkNodeType$SubSup_getInstance() {
    TexTalkNodeType_initFields();
    return TexTalkNodeType$SubSup_instance;
  }
  var TexTalkNodeType$Parameters_instance;
  function TexTalkNodeType$Parameters_getInstance() {
    TexTalkNodeType_initFields();
    return TexTalkNodeType$Parameters_instance;
  }
  var TexTalkNodeType$Comma_instance;
  function TexTalkNodeType$Comma_getInstance() {
    TexTalkNodeType_initFields();
    return TexTalkNodeType$Comma_instance;
  }
  var TexTalkNodeType$Is_instance;
  function TexTalkNodeType$Is_getInstance() {
    TexTalkNodeType_initFields();
    return TexTalkNodeType$Is_instance;
  }
  var TexTalkNodeType$ColonEquals_instance;
  function TexTalkNodeType$ColonEquals_getInstance() {
    TexTalkNodeType_initFields();
    return TexTalkNodeType$ColonEquals_instance;
  }
  TexTalkNodeType.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'TexTalkNodeType',
    interfaces: [Enum]
  };
  function TexTalkNodeType$values() {
    return [TexTalkNodeType$Token_getInstance(), TexTalkNodeType$Identifier_getInstance(), TexTalkNodeType$Operator_getInstance(), TexTalkNodeType$ParenGroup_getInstance(), TexTalkNodeType$SquareGroup_getInstance(), TexTalkNodeType$CurlyGroup_getInstance(), TexTalkNodeType$NamedGroup_getInstance(), TexTalkNodeType$Command_getInstance(), TexTalkNodeType$CommandPart_getInstance(), TexTalkNodeType$Expression_getInstance(), TexTalkNodeType$SubSup_getInstance(), TexTalkNodeType$Parameters_getInstance(), TexTalkNodeType$Comma_getInstance(), TexTalkNodeType$Is_getInstance(), TexTalkNodeType$ColonEquals_getInstance()];
  }
  TexTalkNodeType.values = TexTalkNodeType$values;
  function TexTalkNodeType$valueOf(name) {
    switch (name) {
      case 'Token':
        return TexTalkNodeType$Token_getInstance();
      case 'Identifier':
        return TexTalkNodeType$Identifier_getInstance();
      case 'Operator':
        return TexTalkNodeType$Operator_getInstance();
      case 'ParenGroup':
        return TexTalkNodeType$ParenGroup_getInstance();
      case 'SquareGroup':
        return TexTalkNodeType$SquareGroup_getInstance();
      case 'CurlyGroup':
        return TexTalkNodeType$CurlyGroup_getInstance();
      case 'NamedGroup':
        return TexTalkNodeType$NamedGroup_getInstance();
      case 'Command':
        return TexTalkNodeType$Command_getInstance();
      case 'CommandPart':
        return TexTalkNodeType$CommandPart_getInstance();
      case 'Expression':
        return TexTalkNodeType$Expression_getInstance();
      case 'SubSup':
        return TexTalkNodeType$SubSup_getInstance();
      case 'Parameters':
        return TexTalkNodeType$Parameters_getInstance();
      case 'Comma':
        return TexTalkNodeType$Comma_getInstance();
      case 'Is':
        return TexTalkNodeType$Is_getInstance();
      case 'ColonEquals':
        return TexTalkNodeType$ColonEquals_getInstance();
      default:throwISE('No enum constant mathlingua.common.textalk.TexTalkNodeType.' + name);
    }
  }
  TexTalkNodeType.valueOf_61zpoe$ = TexTalkNodeType$valueOf;
  function TexTalkNode() {
  }
  TexTalkNode.$metadata$ = {
    kind: Kind_INTERFACE,
    simpleName: 'TexTalkNode',
    interfaces: []
  };
  function IsTexTalkNode(lhs, rhs) {
    this.lhs = lhs;
    this.rhs = rhs;
  }
  Object.defineProperty(IsTexTalkNode.prototype, 'type', {
    get: function () {
      return TexTalkNodeType$Is_getInstance();
    }
  });
  IsTexTalkNode.prototype.toCode = function () {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(this.lhs.toCode());
    builder.append_gw00v9$(' is ');
    builder.append_gw00v9$(this.rhs.toCode());
    return builder.toString();
  };
  IsTexTalkNode.prototype.forEach_j2ps96$ = function (fn) {
    fn(this.lhs);
    fn(this.rhs);
  };
  IsTexTalkNode.prototype.transform_7szim8$ = function (transformer) {
    var tmp$, tmp$_0;
    return transformer(new IsTexTalkNode(Kotlin.isType(tmp$ = this.lhs.transform_7szim8$(transformer), ParametersTexTalkNode) ? tmp$ : throwCCE(), Kotlin.isType(tmp$_0 = this.rhs.transform_7szim8$(transformer), ParametersTexTalkNode) ? tmp$_0 : throwCCE()));
  };
  IsTexTalkNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'IsTexTalkNode',
    interfaces: [TexTalkNode]
  };
  IsTexTalkNode.prototype.component1 = function () {
    return this.lhs;
  };
  IsTexTalkNode.prototype.component2 = function () {
    return this.rhs;
  };
  IsTexTalkNode.prototype.copy_bfhfii$ = function (lhs, rhs) {
    return new IsTexTalkNode(lhs === void 0 ? this.lhs : lhs, rhs === void 0 ? this.rhs : rhs);
  };
  IsTexTalkNode.prototype.toString = function () {
    return 'IsTexTalkNode(lhs=' + Kotlin.toString(this.lhs) + (', rhs=' + Kotlin.toString(this.rhs)) + ')';
  };
  IsTexTalkNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.lhs) | 0;
    result = result * 31 + Kotlin.hashCode(this.rhs) | 0;
    return result;
  };
  IsTexTalkNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.lhs, other.lhs) && Kotlin.equals(this.rhs, other.rhs)))));
  };
  function ColonEqualsTexTalkNode(lhs, rhs) {
    this.lhs = lhs;
    this.rhs = rhs;
  }
  Object.defineProperty(ColonEqualsTexTalkNode.prototype, 'type', {
    get: function () {
      return TexTalkNodeType$ColonEquals_getInstance();
    }
  });
  ColonEqualsTexTalkNode.prototype.toCode = function () {
    var builder = StringBuilder_init();
    builder.append_gw00v9$(this.lhs.toCode());
    builder.append_gw00v9$(' := ');
    builder.append_gw00v9$(this.rhs.toCode());
    return builder.toString();
  };
  ColonEqualsTexTalkNode.prototype.forEach_j2ps96$ = function (fn) {
    fn(this.lhs);
    fn(this.rhs);
  };
  ColonEqualsTexTalkNode.prototype.transform_7szim8$ = function (transformer) {
    var tmp$, tmp$_0;
    return transformer(new ColonEqualsTexTalkNode(Kotlin.isType(tmp$ = this.lhs.transform_7szim8$(transformer), ParametersTexTalkNode) ? tmp$ : throwCCE(), Kotlin.isType(tmp$_0 = this.rhs.transform_7szim8$(transformer), ParametersTexTalkNode) ? tmp$_0 : throwCCE()));
  };
  ColonEqualsTexTalkNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ColonEqualsTexTalkNode',
    interfaces: [TexTalkNode]
  };
  ColonEqualsTexTalkNode.prototype.component1 = function () {
    return this.lhs;
  };
  ColonEqualsTexTalkNode.prototype.component2 = function () {
    return this.rhs;
  };
  ColonEqualsTexTalkNode.prototype.copy_bfhfii$ = function (lhs, rhs) {
    return new ColonEqualsTexTalkNode(lhs === void 0 ? this.lhs : lhs, rhs === void 0 ? this.rhs : rhs);
  };
  ColonEqualsTexTalkNode.prototype.toString = function () {
    return 'ColonEqualsTexTalkNode(lhs=' + Kotlin.toString(this.lhs) + (', rhs=' + Kotlin.toString(this.rhs)) + ')';
  };
  ColonEqualsTexTalkNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.lhs) | 0;
    result = result * 31 + Kotlin.hashCode(this.rhs) | 0;
    return result;
  };
  ColonEqualsTexTalkNode.prototype.equals = function (other) {
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
      return TexTalkNodeType$CommandPart_getInstance();
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
  CommandPart.prototype.forEach_j2ps96$ = function (fn) {
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
  CommandPart.prototype.transform_7szim8$ = function (transformer) {
    var tmp$, tmp$_0, tmp$_1, tmp$_2, tmp$_3, tmp$_4, tmp$_5, tmp$_6;
    tmp$_0 = Kotlin.isType(tmp$ = this.name.transform_7szim8$(transformer), TextTexTalkNode) ? tmp$ : throwCCE();
    tmp$_3 = (tmp$_2 = (tmp$_1 = this.square) != null ? tmp$_1.transform_7szim8$(transformer) : null) == null || Kotlin.isType(tmp$_2, GroupTexTalkNode) ? tmp$_2 : throwCCE();
    tmp$_6 = (tmp$_5 = (tmp$_4 = this.subSup) != null ? tmp$_4.transform_7szim8$(transformer) : null) == null || Kotlin.isType(tmp$_5, SubSupTexTalkNode) ? tmp$_5 : throwCCE();
    var $receiver = this.groups;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$_7;
    tmp$_7 = $receiver.iterator();
    while (tmp$_7.hasNext()) {
      var item = tmp$_7.next();
      var tmp$_8;
      destination.add_11rb$(Kotlin.isType(tmp$_8 = item.transform_7szim8$(transformer), GroupTexTalkNode) ? tmp$_8 : throwCCE());
    }
    var $receiver_0 = this.namedGroups;
    var destination_0 = ArrayList_init_0(collectionSizeOrDefault($receiver_0, 10));
    var tmp$_9;
    tmp$_9 = $receiver_0.iterator();
    while (tmp$_9.hasNext()) {
      var item_0 = tmp$_9.next();
      var tmp$_10;
      destination_0.add_11rb$(Kotlin.isType(tmp$_10 = item_0.transform_7szim8$(transformer), NamedGroupTexTalkNode) ? tmp$_10 : throwCCE());
    }
    return transformer(new CommandPart(tmp$_0, tmp$_3, tmp$_6, destination, destination_0));
  };
  CommandPart.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'CommandPart',
    interfaces: [TexTalkNode]
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
  CommandPart.prototype.copy_6bnsom$ = function (name, square, subSup, groups, namedGroups) {
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
      return TexTalkNodeType$Command_getInstance();
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
  Command.prototype.forEach_j2ps96$ = function (fn) {
    var tmp$;
    tmp$ = this.parts.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  Command.prototype.transform_7szim8$ = function (transformer) {
    var $receiver = this.parts;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      var tmp$_0;
      destination.add_11rb$(Kotlin.isType(tmp$_0 = item.transform_7szim8$(transformer), CommandPart) ? tmp$_0 : throwCCE());
    }
    return transformer(new Command(destination));
  };
  Command.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'Command',
    interfaces: [TexTalkNode]
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
  function ExpressionTexTalkNode(children) {
    this.children = children;
  }
  Object.defineProperty(ExpressionTexTalkNode.prototype, 'type', {
    get: function () {
      return TexTalkNodeType$Expression_getInstance();
    }
  });
  ExpressionTexTalkNode.prototype.toCode = function () {
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
  ExpressionTexTalkNode.prototype.forEach_j2ps96$ = function (fn) {
    var tmp$;
    tmp$ = this.children.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  ExpressionTexTalkNode.prototype.transform_7szim8$ = function (transformer) {
    var $receiver = this.children;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      destination.add_11rb$(item.transform_7szim8$(transformer));
    }
    return transformer(new ExpressionTexTalkNode(destination));
  };
  ExpressionTexTalkNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ExpressionTexTalkNode',
    interfaces: [TexTalkNode]
  };
  ExpressionTexTalkNode.prototype.component1 = function () {
    return this.children;
  };
  ExpressionTexTalkNode.prototype.copy_z23bh2$ = function (children) {
    return new ExpressionTexTalkNode(children === void 0 ? this.children : children);
  };
  ExpressionTexTalkNode.prototype.toString = function () {
    return 'ExpressionTexTalkNode(children=' + Kotlin.toString(this.children) + ')';
  };
  ExpressionTexTalkNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.children) | 0;
    return result;
  };
  ExpressionTexTalkNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.children, other.children))));
  };
  function ParametersTexTalkNode(items) {
    this.items = items;
  }
  Object.defineProperty(ParametersTexTalkNode.prototype, 'type', {
    get: function () {
      return TexTalkNodeType$Parameters_getInstance();
    }
  });
  ParametersTexTalkNode.prototype.toCode = function () {
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
  ParametersTexTalkNode.prototype.forEach_j2ps96$ = function (fn) {
    var tmp$;
    tmp$ = this.items.iterator();
    while (tmp$.hasNext()) {
      var element = tmp$.next();
      fn(element);
    }
  };
  ParametersTexTalkNode.prototype.transform_7szim8$ = function (transformer) {
    var $receiver = this.items;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      var tmp$_0;
      destination.add_11rb$(Kotlin.isType(tmp$_0 = item.transform_7szim8$(transformer), ExpressionTexTalkNode) ? tmp$_0 : throwCCE());
    }
    return transformer(new ParametersTexTalkNode(destination));
  };
  ParametersTexTalkNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'ParametersTexTalkNode',
    interfaces: [TexTalkNode]
  };
  ParametersTexTalkNode.prototype.component1 = function () {
    return this.items;
  };
  ParametersTexTalkNode.prototype.copy_qcojy$ = function (items) {
    return new ParametersTexTalkNode(items === void 0 ? this.items : items);
  };
  ParametersTexTalkNode.prototype.toString = function () {
    return 'ParametersTexTalkNode(items=' + Kotlin.toString(this.items) + ')';
  };
  ParametersTexTalkNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.items) | 0;
    return result;
  };
  ParametersTexTalkNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && Kotlin.equals(this.items, other.items))));
  };
  function GroupTexTalkNode(type, parameters) {
    this.type_7n14oy$_0 = type;
    this.parameters = parameters;
  }
  Object.defineProperty(GroupTexTalkNode.prototype, 'type', {
    get: function () {
      return this.type_7n14oy$_0;
    }
  });
  GroupTexTalkNode.prototype.toCode = function () {
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
  GroupTexTalkNode.prototype.forEach_j2ps96$ = function (fn) {
    fn(this.parameters);
  };
  GroupTexTalkNode.prototype.transform_7szim8$ = function (transformer) {
    var tmp$;
    return transformer(new GroupTexTalkNode(this.type, Kotlin.isType(tmp$ = this.parameters.transform_7szim8$(transformer), ParametersTexTalkNode) ? tmp$ : throwCCE()));
  };
  GroupTexTalkNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'GroupTexTalkNode',
    interfaces: [TexTalkNode]
  };
  GroupTexTalkNode.prototype.component1 = function () {
    return this.type;
  };
  GroupTexTalkNode.prototype.component2 = function () {
    return this.parameters;
  };
  GroupTexTalkNode.prototype.copy_vcutp6$ = function (type, parameters) {
    return new GroupTexTalkNode(type === void 0 ? this.type : type, parameters === void 0 ? this.parameters : parameters);
  };
  GroupTexTalkNode.prototype.toString = function () {
    return 'GroupTexTalkNode(type=' + Kotlin.toString(this.type) + (', parameters=' + Kotlin.toString(this.parameters)) + ')';
  };
  GroupTexTalkNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.type) | 0;
    result = result * 31 + Kotlin.hashCode(this.parameters) | 0;
    return result;
  };
  GroupTexTalkNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.type, other.type) && Kotlin.equals(this.parameters, other.parameters)))));
  };
  function NamedGroupTexTalkNode(name, group) {
    this.name = name;
    this.group = group;
  }
  Object.defineProperty(NamedGroupTexTalkNode.prototype, 'type', {
    get: function () {
      return TexTalkNodeType$NamedGroup_getInstance();
    }
  });
  NamedGroupTexTalkNode.prototype.toCode = function () {
    var buffer = StringBuilder_init();
    buffer.append_gw00v9$(this.name.toCode());
    buffer.append_gw00v9$(this.group.toCode());
    return buffer.toString();
  };
  NamedGroupTexTalkNode.prototype.forEach_j2ps96$ = function (fn) {
    fn(this.name);
    fn(this.group);
  };
  NamedGroupTexTalkNode.prototype.transform_7szim8$ = function (transformer) {
    var tmp$, tmp$_0;
    return transformer(new NamedGroupTexTalkNode(Kotlin.isType(tmp$ = this.name.transform_7szim8$(transformer), TextTexTalkNode) ? tmp$ : throwCCE(), Kotlin.isType(tmp$_0 = this.group.transform_7szim8$(transformer), GroupTexTalkNode) ? tmp$_0 : throwCCE()));
  };
  NamedGroupTexTalkNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'NamedGroupTexTalkNode',
    interfaces: [TexTalkNode]
  };
  NamedGroupTexTalkNode.prototype.component1 = function () {
    return this.name;
  };
  NamedGroupTexTalkNode.prototype.component2 = function () {
    return this.group;
  };
  NamedGroupTexTalkNode.prototype.copy_egec36$ = function (name, group) {
    return new NamedGroupTexTalkNode(name === void 0 ? this.name : name, group === void 0 ? this.group : group);
  };
  NamedGroupTexTalkNode.prototype.toString = function () {
    return 'NamedGroupTexTalkNode(name=' + Kotlin.toString(this.name) + (', group=' + Kotlin.toString(this.group)) + ')';
  };
  NamedGroupTexTalkNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.name) | 0;
    result = result * 31 + Kotlin.hashCode(this.group) | 0;
    return result;
  };
  NamedGroupTexTalkNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.name, other.name) && Kotlin.equals(this.group, other.group)))));
  };
  function SubSupTexTalkNode(sub, sup) {
    this.sub = sub;
    this.sup = sup;
  }
  Object.defineProperty(SubSupTexTalkNode.prototype, 'type', {
    get: function () {
      return TexTalkNodeType$SubSup_getInstance();
    }
  });
  SubSupTexTalkNode.prototype.toCode = function () {
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
  SubSupTexTalkNode.prototype.forEach_j2ps96$ = function (fn) {
    if (this.sub != null) {
      fn(this.sub);
    }
    if (this.sup != null) {
      fn(this.sup);
    }
  };
  SubSupTexTalkNode.prototype.transform_7szim8$ = function (transformer) {
    var tmp$, tmp$_0, tmp$_1, tmp$_2;
    return transformer(new SubSupTexTalkNode((tmp$_0 = (tmp$ = this.sub) != null ? tmp$.transform_7szim8$(transformer) : null) == null || Kotlin.isType(tmp$_0, GroupTexTalkNode) ? tmp$_0 : throwCCE(), (tmp$_2 = (tmp$_1 = this.sup) != null ? tmp$_1.transform_7szim8$(transformer) : null) == null || Kotlin.isType(tmp$_2, GroupTexTalkNode) ? tmp$_2 : throwCCE()));
  };
  SubSupTexTalkNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'SubSupTexTalkNode',
    interfaces: [TexTalkNode]
  };
  SubSupTexTalkNode.prototype.component1 = function () {
    return this.sub;
  };
  SubSupTexTalkNode.prototype.component2 = function () {
    return this.sup;
  };
  SubSupTexTalkNode.prototype.copy_r2lyxa$ = function (sub, sup) {
    return new SubSupTexTalkNode(sub === void 0 ? this.sub : sub, sup === void 0 ? this.sup : sup);
  };
  SubSupTexTalkNode.prototype.toString = function () {
    return 'SubSupTexTalkNode(sub=' + Kotlin.toString(this.sub) + (', sup=' + Kotlin.toString(this.sup)) + ')';
  };
  SubSupTexTalkNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.sub) | 0;
    result = result * 31 + Kotlin.hashCode(this.sup) | 0;
    return result;
  };
  SubSupTexTalkNode.prototype.equals = function (other) {
    return this === other || (other !== null && (typeof other === 'object' && (Object.getPrototypeOf(this) === Object.getPrototypeOf(other) && (Kotlin.equals(this.sub, other.sub) && Kotlin.equals(this.sup, other.sup)))));
  };
  function TextTexTalkNode(type, text) {
    this.type_twguqo$_0 = type;
    this.text = text;
  }
  Object.defineProperty(TextTexTalkNode.prototype, 'type', {
    get: function () {
      return this.type_twguqo$_0;
    }
  });
  TextTexTalkNode.prototype.toCode = function () {
    return this.text;
  };
  TextTexTalkNode.prototype.forEach_j2ps96$ = function (fn) {
  };
  TextTexTalkNode.prototype.transform_7szim8$ = function (transformer) {
    return transformer(this);
  };
  TextTexTalkNode.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'TextTexTalkNode',
    interfaces: [TexTalkNode]
  };
  TextTexTalkNode.prototype.component1 = function () {
    return this.type;
  };
  TextTexTalkNode.prototype.component2 = function () {
    return this.text;
  };
  TextTexTalkNode.prototype.copy_buyp7d$ = function (type, text) {
    return new TextTexTalkNode(type === void 0 ? this.type : type, text === void 0 ? this.text : text);
  };
  TextTexTalkNode.prototype.toString = function () {
    return 'TextTexTalkNode(type=' + Kotlin.toString(this.type) + (', text=' + Kotlin.toString(this.text)) + ')';
  };
  TextTexTalkNode.prototype.hashCode = function () {
    var result = 0;
    result = result * 31 + Kotlin.hashCode(this.type) | 0;
    result = result * 31 + Kotlin.hashCode(this.text) | 0;
    return result;
  };
  TextTexTalkNode.prototype.equals = function (other) {
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
      return TexTalkNodeType$Token_getInstance();
    }
  });
  TexTalkToken.prototype.toCode = function () {
    return this.text;
  };
  TexTalkToken.prototype.forEach_j2ps96$ = function (fn) {
  };
  TexTalkToken.prototype.transform_7szim8$ = function (transformer) {
    return transformer(this);
  };
  TexTalkToken.$metadata$ = {
    kind: Kind_CLASS,
    simpleName: 'TexTalkToken',
    interfaces: [TexTalkNode]
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
  function getAncestry(root, node) {
    var path = ArrayList_init();
    getAncestryImpl(root, node, path);
    if (!path.isEmpty()) {
      path.removeAt_za3lpa$(path.size - 1 | 0);
    }
    return reversed(path);
  }
  function getAncestryImpl$lambda(closure$path, closure$node) {
    return function (it) {
      if (closure$path.isEmpty() || !equals(last(closure$path), closure$node)) {
        getAncestryImpl(it, closure$node, closure$path);
      }
      return Unit;
    };
  }
  function getAncestryImpl(root, node, path) {
    if (equals(root, node)) {
      path.add_11rb$(node);
      return;
    }
    path.add_11rb$(root);
    root.forEach_j2ps96$(getAncestryImpl$lambda(path, node));
    if (path.isEmpty() || !equals(last(path), node)) {
      path.removeAt_za3lpa$(path.size - 1 | 0);
    }
  }
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
    this.errors_rts390$_0 = ArrayList_init();
    this.tokens_0 = ArrayList_init();
    this.index_0 = 0;
    var tmp$, tmp$_0, tmp$_1, tmp$_2;
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
       else if (c === 63) {
        this.tokens_0.add_11rb$(new TexTalkToken(String.fromCharCode(c), TexTalkTokenType$Identifier_getInstance(), line, column));
      }
       else if (this.isIdentifierChar_0(c)) {
        var id = new StringBuilder('' + String.fromCharCode(toBoxedChar(c)));
        while (i < text.length && this.isIdentifierChar_0(text.charCodeAt(i))) {
          id.append_s8itvh$(text.charCodeAt((tmp$_0 = i, i = tmp$_0 + 1 | 0, tmp$_0)));
          column = column + 1 | 0;
        }
        if (i < text.length && text.charCodeAt(i) === 63) {
          id.append_s8itvh$(text.charCodeAt((tmp$_1 = i, i = tmp$_1 + 1 | 0, tmp$_1)));
          column = column + 1 | 0;
        }
        this.tokens_0.add_11rb$(new TexTalkToken(id.toString(), TexTalkTokenType$Identifier_getInstance(), line, column));
      }
       else if (this.isOpChar_0(c)) {
        var op = new StringBuilder('' + String.fromCharCode(toBoxedChar(c)));
        while (i < text.length && this.isOpChar_0(text.charCodeAt(i))) {
          op.append_s8itvh$(text.charCodeAt((tmp$_2 = i, i = tmp$_2 + 1 | 0, tmp$_2)));
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
  TexTalkLexerImpl.prototype.isIdentifierChar_0 = function (c) {
    return Regex_init('[$#a-zA-Z0-9]+').matches_6bul2c$(String.fromCharCode(c));
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
  TexTalkParseResult.prototype.copy_9pmc8w$ = function (root, errors) {
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
  var INVALID_0;
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
    this.errors_0 = ArrayList_init();
  }
  TexTalkParserImpl$ParserWorker.prototype.getErrors = function () {
    return this.errors_0;
  };
  TexTalkParserImpl$ParserWorker.prototype.parse = function () {
    var tmp$, tmp$_0;
    var exp = (tmp$ = this.expression_0(null)) != null ? tmp$ : new ExpressionTexTalkNode(emptyList());
    return Kotlin.isType(tmp$_0 = this.resolveColonEqualsNode_0(this.resolveIsNode_0(exp)), ExpressionTexTalkNode) ? tmp$_0 : throwCCE();
  };
  TexTalkParserImpl$ParserWorker.prototype.resolveIsNode_0 = function (texTalkNode) {
    var tmp$;
    if (!Kotlin.isType(texTalkNode, ExpressionTexTalkNode)) {
      return texTalkNode;
    }
    var isIndex = -1;
    tmp$ = texTalkNode.children;
    for (var i = 0; i !== tmp$.size; ++i) {
      var child = texTalkNode.children.get_za3lpa$(i);
      if (Kotlin.isType(child, TextTexTalkNode) && child.type === TexTalkNodeType$Is_getInstance()) {
        if (isIndex < 0) {
          isIndex = i;
        }
         else {
          this.addError_0("A statement can only contain one 'is' statement");
        }
      }
    }
    if (isIndex < 0) {
      return texTalkNode;
    }
    var lhs = this.parameters_0(texTalkNode.children, 0, isIndex);
    var rhs = this.parameters_0(texTalkNode.children, isIndex + 1 | 0, texTalkNode.children.size);
    return new ExpressionTexTalkNode(listOf(new IsTexTalkNode(lhs, rhs)));
  };
  TexTalkParserImpl$ParserWorker.prototype.resolveColonEqualsNode_0 = function (texTalkNode) {
    var tmp$;
    if (!Kotlin.isType(texTalkNode, ExpressionTexTalkNode)) {
      return texTalkNode;
    }
    var colonEqualsIndex = -1;
    tmp$ = texTalkNode.children;
    for (var i = 0; i !== tmp$.size; ++i) {
      var child = texTalkNode.children.get_za3lpa$(i);
      if (Kotlin.isType(child, TextTexTalkNode) && child.type === TexTalkNodeType$ColonEquals_getInstance()) {
        if (colonEqualsIndex < 0) {
          colonEqualsIndex = i;
        }
         else {
          this.addError_0("A statement can only contain one ':='");
        }
      }
    }
    if (colonEqualsIndex < 0) {
      return texTalkNode;
    }
    var lhs = this.parameters_0(texTalkNode.children, 0, colonEqualsIndex);
    var rhs = this.parameters_0(texTalkNode.children, colonEqualsIndex + 1 | 0, texTalkNode.children.size);
    return new ExpressionTexTalkNode(listOf(new ColonEqualsTexTalkNode(lhs, rhs)));
  };
  TexTalkParserImpl$ParserWorker.prototype.parameters_0 = function (texTalkNodes, startInc, endEx) {
    var tmp$;
    var parts = ArrayList_init();
    var i = startInc;
    while (i < endEx) {
      var items = ArrayList_init();
      while (i < endEx && texTalkNodes.get_za3lpa$(i).type !== TexTalkNodeType$Comma_getInstance()) {
        items.add_11rb$(this.resolveIsNode_0(texTalkNodes.get_za3lpa$((tmp$ = i, i = tmp$ + 1 | 0, tmp$))));
      }
      if (i < endEx && texTalkNodes.get_za3lpa$(i).type !== TexTalkNodeType$Comma_getInstance()) {
        this.addError_0('Expected a Comma but found ' + texTalkNodes.get_za3lpa$(i).type);
      }
       else {
        i = i + 1 | 0;
      }
      parts.add_11rb$(new ExpressionTexTalkNode(items));
    }
    return new ParametersTexTalkNode(parts);
  };
  TexTalkParserImpl$ParserWorker.prototype.command_0 = function () {
    if (!this.has_0(TexTalkTokenType$Backslash_getInstance())) {
      return null;
    }
    var backSlash = this.expect_0(TexTalkTokenType$Backslash_getInstance());
    var parts = ArrayList_init();
    while (this.hasNext_0()) {
      var part = this.commandPart_0();
      if (part == null) {
        this.addError_1('Missing a command part', backSlash);
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
      this.addError_1('Expected at least one command part', backSlash);
    }
    return new Command(parts);
  };
  TexTalkParserImpl$ParserWorker.prototype.commandPart_0 = function () {
    if (!this.has_0(TexTalkTokenType$Identifier_getInstance())) {
      return null;
    }
    var name = this.text_0(TexTalkTokenType$Identifier_getInstance(), TexTalkNodeType$Identifier_getInstance());
    var square = this.group_0(TexTalkNodeType$SquareGroup_getInstance());
    var subSup = this.subSup_0();
    var groups = ArrayList_init();
    var startGroup = null;
    var paren = this.group_0(TexTalkNodeType$ParenGroup_getInstance());
    if (paren != null) {
      startGroup = paren;
    }
    if (startGroup == null) {
      var curly = this.group_0(TexTalkNodeType$CurlyGroup_getInstance());
      if (curly != null) {
        startGroup = curly;
      }
    }
    if (startGroup != null) {
      groups.add_11rb$(startGroup);
      while (this.hasNext_0()) {
        var grp = this.group_0(startGroup.type);
        if (grp == null)
          break;
        groups.add_11rb$(grp);
      }
    }
    var namedGroups = ArrayList_init();
    if (this.has_0(TexTalkTokenType$Colon_getInstance())) {
      this.expect_0(TexTalkTokenType$Colon_getInstance());
      while (this.hasNext_0()) {
        var namedGrp = this.namedGroup_0();
        if (namedGrp == null)
          break;
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
      tmp$ = new SubSupTexTalkNode(sub, sup);
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
    var curly = this.group_0(TexTalkNodeType$CurlyGroup_getInstance());
    if (curly != null) {
      grp = curly;
    }
    if (grp == null) {
      var paren = this.group_0(TexTalkNodeType$ParenGroup_getInstance());
      if (paren != null) {
        grp = paren;
      }
    }
    if (grp == null) {
      this.addError_2('Expected a value with an underscore', row, column);
      grp = new GroupTexTalkNode(TexTalkNodeType$CurlyGroup_getInstance(), new ParametersTexTalkNode(emptyList()));
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
    var curly = this.group_0(TexTalkNodeType$CurlyGroup_getInstance());
    if (curly != null) {
      grp = curly;
    }
    if (grp == null) {
      var paren = this.group_0(TexTalkNodeType$ParenGroup_getInstance());
      if (paren != null) {
        grp = paren;
      }
    }
    if (grp == null) {
      this.addError_2('Expected a value with a caret', row, column);
      grp = new GroupTexTalkNode(TexTalkNodeType$CurlyGroup_getInstance(), new ParametersTexTalkNode(emptyList()));
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
      this.next_0();
      var exp = this.expression_0(terminators);
      if (exp == null)
        break;
      expressions.add_11rb$(exp);
    }
    this.expect_0(endType);
    return new GroupTexTalkNode(nodeType, new ParametersTexTalkNode(expressions));
  };
  TexTalkParserImpl$ParserWorker.prototype.namedGroup_0 = function () {
    var tmp$, tmp$_0;
    if (!this.hasHas_0(TexTalkTokenType$Identifier_getInstance(), TexTalkTokenType$LCurly_getInstance())) {
      return null;
    }
    var rawText = this.text_0(TexTalkTokenType$Identifier_getInstance(), TexTalkNodeType$Identifier_getInstance());
    if (rawText != null) {
      tmp$ = rawText;
    }
     else {
      this.addError_0('Expected an identifier in a named group');
      tmp$ = new TextTexTalkNode(TexTalkNodeType$Identifier_getInstance(), 'INVALID');
    }
    var text = tmp$;
    var rawGroup = this.group_0(TexTalkNodeType$CurlyGroup_getInstance());
    if (rawGroup != null) {
      tmp$_0 = rawGroup;
    }
     else {
      this.addError_0('Expected a group in a named group');
      tmp$_0 = new GroupTexTalkNode(TexTalkNodeType$CurlyGroup_getInstance(), new ParametersTexTalkNode(emptyList()));
    }
    var group = tmp$_0;
    return new NamedGroupTexTalkNode(text, group);
  };
  TexTalkParserImpl$ParserWorker.prototype.text_0 = function (tokenType, nodeType) {
    var tmp$;
    if (!this.has_0(tokenType)) {
      tmp$ = null;
    }
     else
      tmp$ = new TextTexTalkNode(nodeType, this.next_0().text);
    return tmp$;
  };
  TexTalkParserImpl$ParserWorker.prototype.expression_0 = function (terminators) {
    var tmp$, tmp$_0, tmp$_1, tmp$_2, tmp$_3, tmp$_4, tmp$_5, tmp$_6, tmp$_7, tmp$_8;
    var nodes = ArrayList_init();
    while (this.hasNext_0() && (terminators == null || !terminators.contains_11rb$(this.texTalkLexer_0.peek().tokenType))) {
      var child = (tmp$_7 = (tmp$_6 = (tmp$_5 = (tmp$_4 = (tmp$_3 = (tmp$_2 = (tmp$_1 = (tmp$_0 = (tmp$ = this.command_0()) != null ? tmp$ : this.group_0(TexTalkNodeType$ParenGroup_getInstance())) != null ? tmp$_0 : this.group_0(TexTalkNodeType$CurlyGroup_getInstance())) != null ? tmp$_1 : this.text_0(TexTalkTokenType$Is_getInstance(), TexTalkNodeType$Is_getInstance())) != null ? tmp$_2 : this.text_0(TexTalkTokenType$Identifier_getInstance(), TexTalkNodeType$Identifier_getInstance())) != null ? tmp$_3 : this.text_0(TexTalkTokenType$Operator_getInstance(), TexTalkNodeType$Operator_getInstance())) != null ? tmp$_4 : this.text_0(TexTalkTokenType$Comma_getInstance(), TexTalkNodeType$Comma_getInstance())) != null ? tmp$_5 : this.text_0(TexTalkTokenType$Caret_getInstance(), TexTalkNodeType$Operator_getInstance())) != null ? tmp$_6 : this.text_0(TexTalkTokenType$Underscore_getInstance(), TexTalkNodeType$Operator_getInstance())) != null ? tmp$_7 : this.text_0(TexTalkTokenType$ColonEquals_getInstance(), TexTalkNodeType$ColonEquals_getInstance());
      if (child == null) {
        var peek = this.texTalkLexer_0.peek();
        this.addError_1('Unexpected token ' + peek.text, peek);
        this.next_0();
      }
       else {
        nodes.add_11rb$(child);
      }
    }
    if (nodes.isEmpty()) {
      tmp$_8 = null;
    }
     else
      tmp$_8 = new ExpressionTexTalkNode(nodes);
    return tmp$_8;
  };
  TexTalkParserImpl$ParserWorker.prototype.expect_0 = function (tokenType) {
    var tmp$;
    if (this.has_0(tokenType)) {
      return this.next_0();
    }
     else {
      if (this.hasNext_0()) {
        tmp$ = "Expected a token of type '" + tokenType + "' but found type " + ("'" + this.texTalkLexer_0.peek().type + "' for text '" + this.texTalkLexer_0.peek().text + "' ") + ('(Line: ' + (this.texTalkLexer_0.peek().row + 1 | 0) + ', Column: ' + (this.texTalkLexer_0.peek().column + 1 | 0) + ')');
      }
       else {
        tmp$ = 'Expected a token of type ' + tokenType + ' but found the end of input';
      }
      var message = tmp$;
      this.addError_0(message);
      return INVALID_0;
    }
  };
  TexTalkParserImpl$ParserWorker.prototype.hasNext_0 = function () {
    return this.texTalkLexer_0.hasNext();
  };
  TexTalkParserImpl$ParserWorker.prototype.next_0 = function () {
    return this.texTalkLexer_0.next();
  };
  TexTalkParserImpl$ParserWorker.prototype.has_0 = function (tokenType) {
    return this.hasNext_0() && this.texTalkLexer_0.peek().tokenType === tokenType;
  };
  TexTalkParserImpl$ParserWorker.prototype.hasHas_0 = function (tokenType1, tokenType2) {
    return this.has_0(tokenType1) && this.texTalkLexer_0.hasNextNext() && this.texTalkLexer_0.peekPeek().tokenType === tokenType2;
  };
  TexTalkParserImpl$ParserWorker.prototype.addError_1 = function (message, token) {
    this.addError_2(message, token.row, token.column);
  };
  TexTalkParserImpl$ParserWorker.prototype.addError_0 = function (message) {
    this.addError_2(message, -1, -1);
  };
  TexTalkParserImpl$ParserWorker.prototype.addError_2 = function (message, row, column) {
    this.errors_0.add_11rb$(new ParseError(message, row, column));
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
  function findCommands(texTalkNode) {
    var commands = ArrayList_init();
    findCommandsImpl(texTalkNode, commands);
    return distinct(commands);
  }
  function replaceSignatures$lambda(closure$signature, closure$replacement, closure$texTalkNode) {
    return function (it) {
      if (Kotlin.isType(it, Command) && equals(getCommandSignature(it).toCode(), closure$signature)) {
        return new TextTexTalkNode(TexTalkNodeType$Identifier_getInstance(), closure$replacement);
      }
       else {
        return closure$texTalkNode;
      }
    };
  }
  function replaceSignatures(texTalkNode, signature, replacement) {
    return texTalkNode.transform_7szim8$(replaceSignatures$lambda(signature, replacement, texTalkNode));
  }
  function replaceCommands$lambda(closure$shouldProcessChalk, closure$cmdToReplacement, closure$shouldProcessTex) {
    return function (it) {
      var tmp$;
      if (!closure$shouldProcessChalk(it) || !Kotlin.isType(it, Statement)) {
        return it;
      }
       else {
        var validation = it.texTalkRoot;
        if (Kotlin.isType(validation, ValidationFailure))
          return it;
        else if (Kotlin.isType(validation, ValidationSuccess)) {
          var root = validation.value;
          var newRoot = Kotlin.isType(tmp$ = replaceCommands_0(root, root, closure$cmdToReplacement, closure$shouldProcessTex), ExpressionTexTalkNode) ? tmp$ : throwCCE();
          return new Statement(newRoot.toCode(), new ValidationSuccess(newRoot));
        }
         else
          return Kotlin.noWhenBranchMatched();
      }
    };
  }
  function replaceCommands(node, cmdToReplacement, shouldProcessChalk, shouldProcessTex) {
    return node.transform_nrl0ww$(replaceCommands$lambda(shouldProcessChalk, cmdToReplacement, shouldProcessTex));
  }
  function replaceCommands$lambda_0(closure$shouldProcess, closure$root, closure$cmdToReplacement) {
    return function (it) {
      if (!closure$shouldProcess(closure$root, it) || !Kotlin.isType(it, Command)) {
        return it;
      }
       else {
        if (!closure$cmdToReplacement.containsKey_11rb$(it)) {
          return it;
        }
         else {
          var name = closure$cmdToReplacement.get_11rb$(it);
          return new TextTexTalkNode(TexTalkNodeType$Identifier_getInstance(), ensureNotNull(name));
        }
      }
    };
  }
  function replaceCommands_0(texTalkNode, root, cmdToReplacement, shouldProcess) {
    return texTalkNode.transform_7szim8$(replaceCommands$lambda_0(shouldProcess, root, cmdToReplacement));
  }
  function findCommandsImpl$lambda(closure$commands) {
    return function (it) {
      findCommandsImpl(it, closure$commands);
      return Unit;
    };
  }
  function findCommandsImpl(texTalkNode, commands) {
    if (Kotlin.isType(texTalkNode, Command)) {
      commands.add_11rb$(texTalkNode);
    }
    texTalkNode.forEach_j2ps96$(findCommandsImpl$lambda(commands));
  }
  function separateIsStatements$lambda(it) {
    var tmp$;
    if (Kotlin.isType(it, ClauseListNode)) {
      var newClauses = ArrayList_init();
      tmp$ = it.clauses.iterator();
      while (tmp$.hasNext()) {
        var clause = tmp$.next();
        if (Kotlin.isType(clause, Statement)) {
          var separated = findSeparatedIsNodes(clause);
          if (separated == null) {
            newClauses.add_11rb$(clause);
          }
           else {
            var destination = ArrayList_init_0(collectionSizeOrDefault(separated, 10));
            var tmp$_0;
            tmp$_0 = separated.iterator();
            while (tmp$_0.hasNext()) {
              var item = tmp$_0.next();
              var tmp$_1 = destination.add_11rb$;
              var root = new ExpressionTexTalkNode(listOf(item));
              tmp$_1.call(destination, new Statement(root.toCode(), new ValidationSuccess(root)));
            }
            newClauses.addAll_brywnq$(destination);
          }
        }
         else {
          newClauses.add_11rb$(clause);
        }
      }
      return new ClauseListNode(newClauses);
    }
     else {
      return it;
    }
  }
  function separateIsStatements(node) {
    return node.transform_nrl0ww$(separateIsStatements$lambda);
  }
  function findSeparatedIsNodes(node) {
    var tmp$, tmp$_0, tmp$_1;
    var validation = node.texTalkRoot;
    if (Kotlin.isType(validation, ValidationFailure))
      tmp$_1 = null;
    else if (Kotlin.isType(validation, ValidationSuccess)) {
      var root = validation.value;
      if (root.children.size === 1 && Kotlin.isType(root.children.get_za3lpa$(0), IsTexTalkNode)) {
        var isNode = Kotlin.isType(tmp$ = root.children.get_za3lpa$(0), IsTexTalkNode) ? tmp$ : throwCCE();
        tmp$_0 = separateIsStatementsUnder(isNode);
      }
       else {
        tmp$_0 = null;
      }
      return tmp$_0;
    }
     else
      tmp$_1 = Kotlin.noWhenBranchMatched();
    return tmp$_1;
  }
  function separateIsStatementsUnder(isNode) {
    var tmp$, tmp$_0;
    var result = ArrayList_init();
    tmp$ = isNode.lhs.items.iterator();
    while (tmp$.hasNext()) {
      var left = tmp$.next();
      tmp$_0 = isNode.rhs.items.iterator();
      while (tmp$_0.hasNext()) {
        var right = tmp$_0.next();
        result.add_11rb$(new IsTexTalkNode(new ParametersTexTalkNode(listOf(left)), new ParametersTexTalkNode(listOf(right))));
      }
    }
    return result;
  }
  function glueCommands$lambda(it) {
    var tmp$;
    var tmp$_0 = Kotlin.isType(it, Statement) && Kotlin.isType(it.texTalkRoot, ValidationSuccess);
    if (tmp$_0) {
      var $receiver = it.texTalkRoot.value.children;
      var all$result;
      all$break: do {
        var tmp$_1;
        if (Kotlin.isType($receiver, Collection) && $receiver.isEmpty()) {
          all$result = true;
          break all$break;
        }
        tmp$_1 = $receiver.iterator();
        while (tmp$_1.hasNext()) {
          var element = tmp$_1.next();
          if (!Kotlin.isType(element, Command)) {
            all$result = false;
            break all$break;
          }
        }
        all$result = true;
      }
       while (false);
      tmp$_0 = all$result;
    }
    if (tmp$_0) {
      var exp = it.texTalkRoot.value;
      var cmds = getCommandsToGlue(exp);
      var gluedCmds = glueCommands_0(cmds);
      if (gluedCmds.size !== 1) {
        throw Error_init('Expected id ' + it + ' to only contain a single glued command');
      }
      var newExp = new ExpressionTexTalkNode(listOf(gluedCmds.get_za3lpa$(0)));
      return new Statement(newExp.toCode(), new ValidationSuccess(newExp));
    }
     else if (Kotlin.isType(it, Statement) && Kotlin.isType(it.texTalkRoot, ValidationSuccess) && it.texTalkRoot.value.children.size === 1 && Kotlin.isType(it.texTalkRoot.value.children.get_za3lpa$(0), IsTexTalkNode)) {
      var isNode = Kotlin.isType(tmp$ = it.texTalkRoot.value.children.get_za3lpa$(0), IsTexTalkNode) ? tmp$ : throwCCE();
      if (isNode.rhs.items.size !== 1) {
        throw Error_init("Expected 'is' node " + isNode + ' to only contain a single rhs item');
      }
      var cmds_0 = getCommandsToGlue(isNode.rhs.items.get_za3lpa$(0));
      var gluedCmds_0 = glueCommands_0(cmds_0);
      if (gluedCmds_0.size !== 1) {
        throw Error_init("Expected 'is' node " + isNode + ' to have only one glued rhs command');
      }
      var newExp_0 = new ExpressionTexTalkNode(listOf(new IsTexTalkNode(isNode.lhs, new ParametersTexTalkNode(listOf(new ExpressionTexTalkNode(listOf(gluedCmds_0.get_za3lpa$(0))))))));
      return new Statement(newExp_0.toCode(), new ValidationSuccess(newExp_0));
    }
     else {
      return it;
    }
  }
  function glueCommands(node) {
    return node.transform_nrl0ww$(glueCommands$lambda);
  }
  function getCommandsToGlue(node) {
    var tmp$;
    var cmds = ArrayList_init();
    tmp$ = node.children.iterator();
    while (tmp$.hasNext()) {
      var n = tmp$.next();
      if (!Kotlin.isType(n, Command)) {
        throw Error_init('Unexpected non-Command node');
      }
      cmds.add_11rb$(n);
    }
    return glueCommands_0(cmds);
  }
  function glueCommands_0(commands) {
    var tmp$;
    if (commands.isEmpty()) {
      return emptyList();
    }
    if (commands.size === 1) {
      return listOf(first(commands));
    }
    var last_0 = last(commands);
    var newCommands = ArrayList_init();
    tmp$ = commands.size - 1 | 0;
    for (var i = 0; i < tmp$; i++) {
      var cmd = commands.get_za3lpa$(i);
      var parts = ArrayList_init();
      parts.addAll_brywnq$(cmd.parts);
      parts.addAll_brywnq$(last_0.parts);
      newCommands.add_11rb$(new Command(parts));
    }
    return newCommands;
  }
  function getSignature(stmt) {
    var tmp$;
    var sigs = findAllStatementSignatures(stmt);
    if (sigs.size === 1) {
      tmp$ = first_0(sigs);
    }
     else
      tmp$ = null;
    return tmp$;
  }
  function findAllStatementSignatures(stmt) {
    var tmp$;
    var rootValidation = stmt.texTalkRoot;
    if (Kotlin.isType(rootValidation, ValidationSuccess)) {
      var expressionNode = rootValidation.value;
      var signatures = LinkedHashSet_init();
      findAllSignaturesImpl_0(expressionNode, signatures);
      tmp$ = signatures;
    }
     else if (Kotlin.isType(rootValidation, ValidationFailure))
      return emptySet();
    else
      tmp$ = Kotlin.noWhenBranchMatched();
    return tmp$;
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
  function locateAllSignatures(node) {
    var signatures = LinkedHashSet_init();
    findAllSignaturesImpl(node, signatures);
    return signatures;
  }
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
  function findAllSignaturesImpl$lambda_0(closure$signatures) {
    return function (it) {
      findAllSignaturesImpl_0(it, closure$signatures);
      return Unit;
    };
  }
  function findAllSignaturesImpl_0(texTalkNode, signatures) {
    var tmp$;
    if (Kotlin.isType(texTalkNode, IsTexTalkNode)) {
      tmp$ = texTalkNode.rhs.items.iterator();
      while (tmp$.hasNext()) {
        var expNode = tmp$.next();
        var sig = getMergedCommandSignature(expNode);
        if (sig != null) {
          signatures.add_11rb$(sig);
        }
      }
      return;
    }
     else if (Kotlin.isType(texTalkNode, Command)) {
      var sig_0 = getCommandSignature(texTalkNode).toCode();
      signatures.add_11rb$(sig_0);
    }
    texTalkNode.forEach_j2ps96$(findAllSignaturesImpl$lambda_0(signatures));
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
    return new SubSupTexTalkNode(callOrNull(node.sub, getCallableRef('getGroupNodeForSignature', function (node) {
      return getGroupNodeForSignature(node);
    })), callOrNull(node.sup, getCallableRef('getGroupNodeForSignature', function (node) {
      return getGroupNodeForSignature(node);
    })));
  }
  function getGroupNodeForSignature(node) {
    return new GroupTexTalkNode(node.type, getParametersNodeForSignature(node.parameters));
  }
  function getParametersNodeForSignature(node) {
    var $receiver = node.items;
    var destination = ArrayList_init_0(collectionSizeOrDefault($receiver, 10));
    var tmp$;
    tmp$ = $receiver.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      destination.add_11rb$(new ExpressionTexTalkNode(listOf(new TextTexTalkNode(TexTalkNodeType$Identifier_getInstance(), '?'))));
    }
    return new ParametersTexTalkNode(destination);
  }
  function getNamedGroupNodeForSignature(node) {
    return new NamedGroupTexTalkNode(node.name, getGroupNodeForSignature(node.group));
  }
  function moveInlineCommandsToIsNode$realShouldProcessTex(closure$shouldProcessTex, closure$knownDefSigs) {
    return function (root, node) {
      var tmp$;
      if (!closure$shouldProcessTex(root, node)) {
        return false;
      }
      if (Kotlin.isType(node, Command) && !closure$knownDefSigs.contains_11rb$(getCommandSignature(node).toCode())) {
        return false;
      }
      var parents = getAncestry(root, node);
      tmp$ = parents.iterator();
      while (tmp$.hasNext()) {
        var p = tmp$.next();
        if (Kotlin.isType(p, IsTexTalkNode)) {
          return false;
        }
      }
      return true;
    };
  }
  function moveInlineCommandsToIsNode$lambda(closure$seed, closure$shouldProcessChalk, closure$realShouldProcessTex) {
    return function (it) {
      var tmp$, tmp$_0, tmp$_1, tmp$_2;
      if (Kotlin.isType(it, ClauseListNode)) {
        var newClauses = ArrayList_init();
        tmp$ = it.clauses.iterator();
        while (tmp$.hasNext()) {
          var c = tmp$.next();
          if (Kotlin.isType(c, Statement)) {
            tmp$_1 = (tmp$_0 = closure$seed.v, closure$seed.v = tmp$_0 + 1 | 0, tmp$_0);
            tmp$_2 = getCallableRef('realShouldProcessTex', function (root, node) {
              return closure$realShouldProcessTex(root, node);
            });
            var transformed = moveStatementInlineCommandsToIsNode(tmp$_1, c, closure$shouldProcessChalk, tmp$_2);
            newClauses.add_11rb$(transformed);
          }
           else {
            newClauses.add_11rb$(c);
          }
        }
        return new ClauseListNode(newClauses);
      }
       else {
        return it;
      }
    };
  }
  function moveInlineCommandsToIsNode(defs, node, shouldProcessChalk, shouldProcessTex) {
    var destination = ArrayList_init_0(collectionSizeOrDefault(defs, 10));
    var tmp$;
    tmp$ = defs.iterator();
    while (tmp$.hasNext()) {
      var item = tmp$.next();
      destination.add_11rb$(item.signature);
    }
    var knownDefSigs = toSet(filterNotNull(destination));
    var realShouldProcessTex = moveInlineCommandsToIsNode$realShouldProcessTex(shouldProcessTex, knownDefSigs);
    var seed = {v: 0};
    return node.transform_nrl0ww$(moveInlineCommandsToIsNode$lambda(seed, shouldProcessChalk, realShouldProcessTex));
  }
  function moveStatementInlineCommandsToIsNode$shouldProcessTexNodes(closure$shouldProcessTex) {
    return function (root, node) {
      if (!closure$shouldProcessTex(root, node)) {
        return false;
      }
      var $receiver = getAncestry(root, node);
      var any$result;
      any$break: do {
        var tmp$;
        if (Kotlin.isType($receiver, Collection) && $receiver.isEmpty()) {
          any$result = false;
          break any$break;
        }
        tmp$ = $receiver.iterator();
        while (tmp$.hasNext()) {
          var element = tmp$.next();
          if (Kotlin.isType(element, IsTexTalkNode)) {
            any$result = true;
            break any$break;
          }
        }
        any$result = false;
      }
       while (false);
      return !any$result;
    };
  }
  function moveStatementInlineCommandsToIsNode(seed, stmt, shouldProcessChalk, shouldProcessTex) {
    var tmp$, tmp$_0, tmp$_1, tmp$_2;
    var validation = stmt.texTalkRoot;
    if (Kotlin.isType(validation, ValidationFailure)) {
      return stmt;
    }
    var root = (Kotlin.isType(tmp$ = validation, ValidationSuccess) ? tmp$ : throwCCE()).value;
    if (!shouldProcessChalk(stmt)) {
      return stmt;
    }
    var shouldProcessTexNodes = moveStatementInlineCommandsToIsNode$shouldProcessTexNodes(shouldProcessTex);
    var commandsFound = findCommands(root);
    var cmdToReplacement = LinkedHashMap_init();
    var count = seed;
    tmp$_0 = commandsFound.iterator();
    while (tmp$_0.hasNext()) {
      var cmd = tmp$_0.next();
      if (shouldProcessTex(root, cmd)) {
        var value = '$' + (tmp$_1 = count, count = tmp$_1 + 1 | 0, tmp$_1);
        cmdToReplacement.put_xwzc9p$(cmd, value);
      }
    }
    var cmdsToProcess = cmdToReplacement.keys;
    var newNode = Kotlin.isType(tmp$_2 = replaceCommands(stmt, cmdToReplacement, shouldProcessChalk, getCallableRef('shouldProcessTexNodes', function (root, node) {
      return shouldProcessTexNodes(root, node);
    })), Clause) ? tmp$_2 : throwCCE();
    if (commandsFound.isEmpty()) {
      return stmt;
    }
    if (cmdsToProcess.isEmpty()) {
      return stmt;
    }
    var destination = ArrayList_init_0(collectionSizeOrDefault(cmdsToProcess, 10));
    var tmp$_3;
    tmp$_3 = cmdsToProcess.iterator();
    while (tmp$_3.hasNext()) {
      var item = tmp$_3.next();
      destination.add_11rb$(new Identifier(ensureNotNull(cmdToReplacement.get_11rb$(item))));
    }
    var tmp$_4 = new ForSection(destination);
    var destination_0 = ArrayList_init_0(collectionSizeOrDefault(cmdsToProcess, 10));
    var tmp$_5;
    tmp$_5 = cmdsToProcess.iterator();
    while (tmp$_5.hasNext()) {
      var item_0 = tmp$_5.next();
      var tmp$_6 = destination_0.add_11rb$;
      var isNode = new IsTexTalkNode(new ParametersTexTalkNode(listOf(new ExpressionTexTalkNode(listOf(new TextTexTalkNode(TexTalkNodeType$Identifier_getInstance(), ensureNotNull(cmdToReplacement.get_11rb$(item_0))))))), new ParametersTexTalkNode(listOf(new ExpressionTexTalkNode(listOf(item_0)))));
      tmp$_6.call(destination_0, new Statement(isNode.toCode(), new ValidationSuccess(new ExpressionTexTalkNode(listOf(isNode)))));
    }
    return new ForGroup(tmp$_4, new WhereSection(new ClauseListNode(destination_0)), new ThenSection(new ClauseListNode(listOf(newNode))));
  }
  function replaceRepresents$lambda(it) {
    return true;
  }
  function replaceRepresents$chalkTransformer(closure$filter, closure$repMap) {
    return function (node) {
      var tmp$, tmp$_0, tmp$_1, tmp$_2, tmp$_3, tmp$_4, tmp$_5, tmp$_6, tmp$_7, tmp$_8, tmp$_9, tmp$_10;
      if (!closure$filter(node)) {
        return node;
      }
      if (!Kotlin.isType(node, ClauseListNode)) {
        return node;
      }
      var newClauses = ArrayList_init();
      tmp$ = node.clauses.iterator();
      while (tmp$.hasNext()) {
        var clause = tmp$.next();
        if (!Kotlin.isType(clause, Statement)) {
          newClauses.add_11rb$(clause);
          continue;
        }
        if (Kotlin.isType(clause.texTalkRoot, ValidationSuccess) && clause.texTalkRoot.value.children.size === 1 && Kotlin.isType(clause.texTalkRoot.value.children.get_za3lpa$(0), Command)) {
          var command = Kotlin.isType(tmp$_0 = clause.texTalkRoot.value.children.get_za3lpa$(0), Command) ? tmp$_0 : throwCCE();
          var sig = getCommandSignature(command).toCode();
          if (!closure$repMap.containsKey_11rb$(sig)) {
            return node;
          }
          var rep = ensureNotNull(closure$repMap.get_11rb$(sig));
          var cmdVars = getVars_1(command);
          var defIndirectVars = getRepresentsIdVars(rep);
          var map = LinkedHashMap_init();
          for (var i = 0; i !== cmdVars.size; ++i) {
            var key = defIndirectVars.get_za3lpa$(i);
            var value = cmdVars.get_za3lpa$(i);
            map.put_xwzc9p$(key, value);
          }
          var ifThen = buildIfThen_0(rep);
          if (ifThen.ifSection.clauses.clauses.isEmpty() && ifThen.thenSection.clauses.clauses.size === 1) {
            tmp$_1 = ifThen.thenSection.clauses.clauses.get_za3lpa$(0);
          }
           else {
            tmp$_1 = ifThen;
          }
          var res = tmp$_1;
          newClauses.add_11rb$(Kotlin.isType(tmp$_2 = renameVars_0(res, map), Clause) ? tmp$_2 : throwCCE());
        }
         else if (Kotlin.isType(clause.texTalkRoot, ValidationSuccess) && clause.texTalkRoot.value.children.size === 3 && Kotlin.isType(clause.texTalkRoot.value.children.get_za3lpa$(0), TextTexTalkNode) && Kotlin.isType(clause.texTalkRoot.value.children.get_za3lpa$(1), Command) && Kotlin.isType(clause.texTalkRoot.value.children.get_za3lpa$(2), TextTexTalkNode)) {
          var left = Kotlin.isType(tmp$_3 = clause.texTalkRoot.value.children.get_za3lpa$(0), TextTexTalkNode) ? tmp$_3 : throwCCE();
          var op = Kotlin.isType(tmp$_4 = clause.texTalkRoot.value.children.get_za3lpa$(1), Command) ? tmp$_4 : throwCCE();
          var right = Kotlin.isType(tmp$_5 = clause.texTalkRoot.value.children.get_za3lpa$(2), TextTexTalkNode) ? tmp$_5 : throwCCE();
          var sig_0 = getCommandSignature(op).toCode();
          if (!closure$repMap.containsKey_11rb$(sig_0)) {
            return node;
          }
          var rep_0 = ensureNotNull(closure$repMap.get_11rb$(sig_0));
          var cmdVars_0 = listOf_0([left.text, right.text]);
          if (Kotlin.isType(rep_0.id.texTalkRoot, ValidationFailure)) {
            return node;
          }
          var validation = Kotlin.isType(tmp$_6 = rep_0.id.texTalkRoot, ValidationSuccess) ? tmp$_6 : throwCCE();
          if (validation.value.children.size !== 3 || !Kotlin.isType(validation.value.children.get_za3lpa$(0), TextTexTalkNode) || !Kotlin.isType(validation.value.children.get_za3lpa$(1), Command) || !Kotlin.isType(validation.value.children.get_za3lpa$(2), TextTexTalkNode)) {
            return node;
          }
          var repLeftOpRight = validation.value.children;
          var repLeft = (Kotlin.isType(tmp$_7 = repLeftOpRight.get_za3lpa$(0), TextTexTalkNode) ? tmp$_7 : throwCCE()).text;
          var repRight = (Kotlin.isType(tmp$_8 = repLeftOpRight.get_za3lpa$(2), TextTexTalkNode) ? tmp$_8 : throwCCE()).text;
          var defIndirectVars_0 = listOf_0([repLeft, repRight]);
          var map_0 = LinkedHashMap_init();
          for (var i_0 = 0; i_0 !== cmdVars_0.size; ++i_0) {
            var key_0 = defIndirectVars_0.get_za3lpa$(i_0);
            var value_0 = cmdVars_0.get_za3lpa$(i_0);
            map_0.put_xwzc9p$(key_0, value_0);
          }
          var ifThen_0 = buildIfThen_0(rep_0);
          if (ifThen_0.ifSection.clauses.clauses.isEmpty() && ifThen_0.thenSection.clauses.clauses.size === 1) {
            tmp$_9 = ifThen_0.thenSection.clauses.clauses.get_za3lpa$(0);
          }
           else {
            tmp$_9 = ifThen_0;
          }
          var res_0 = tmp$_9;
          newClauses.add_11rb$(Kotlin.isType(tmp$_10 = renameVars_0(res_0, map_0), Clause) ? tmp$_10 : throwCCE());
        }
         else {
          newClauses.add_11rb$(clause);
        }
      }
      return new ClauseListNode(newClauses);
    };
  }
  function replaceRepresents(node, represents, filter) {
    if (filter === void 0)
      filter = replaceRepresents$lambda;
    var tmp$;
    var repMap = LinkedHashMap_init();
    tmp$ = represents.iterator();
    while (tmp$.hasNext()) {
      var rep = tmp$.next();
      var sig = rep.signature;
      if (sig != null) {
        repMap.put_xwzc9p$(sig, rep);
      }
    }
    var chalkTransformer = replaceRepresents$chalkTransformer(filter, repMap);
    return node.transform_nrl0ww$(getCallableRef('chalkTransformer', function (node) {
      return chalkTransformer(node);
    }));
  }
  function replaceIsNodes$lambda(it) {
    return true;
  }
  function replaceIsNodes$chalkTransformer(closure$filter, closure$defMap) {
    return function (node) {
      var tmp$, tmp$_0, tmp$_1, tmp$_2;
      if (!closure$filter(node)) {
        return node;
      }
      if (!Kotlin.isType(node, Statement)) {
        return node;
      }
      if (Kotlin.isType(node.texTalkRoot, ValidationFailure) || (Kotlin.isType(tmp$ = node.texTalkRoot, ValidationSuccess) ? tmp$ : throwCCE()).value.children.size !== 1 || !Kotlin.isType(node.texTalkRoot.value.children.get_za3lpa$(0), IsTexTalkNode)) {
        return node;
      }
      var isNode = Kotlin.isType(tmp$_0 = node.texTalkRoot.value.children.get_za3lpa$(0), IsTexTalkNode) ? tmp$_0 : throwCCE();
      if (isNode.rhs.items.size !== 1 || isNode.rhs.items.get_za3lpa$(0).children.size !== 1 || !Kotlin.isType(isNode.rhs.items.get_za3lpa$(0).children.get_za3lpa$(0), Command)) {
        return node;
      }
      var command = Kotlin.isType(tmp$_1 = isNode.rhs.items.get_za3lpa$(0).children.get_za3lpa$(0), Command) ? tmp$_1 : throwCCE();
      var sig = getCommandSignature(command).toCode();
      if (!closure$defMap.containsKey_11rb$(sig)) {
        return node;
      }
      var def = ensureNotNull(closure$defMap.get_11rb$(sig));
      var cmdVars = getVars_1(command);
      var defDirectVars = getDefinesDirectVars(def);
      var defIndirectVars = getDefinesIdVars(def);
      if (cmdVars.size !== defIndirectVars.size) {
        return node;
      }
      var map = LinkedHashMap_init();
      for (var i = 0; i !== cmdVars.size; ++i) {
        var key = defIndirectVars.get_za3lpa$(i);
        var value = cmdVars.get_za3lpa$(i);
        map.put_xwzc9p$(key, value);
      }
      var lhsVars = getVars_1(isNode.lhs);
      if (lhsVars.size > defDirectVars.size) {
        return node;
      }
      for (var i_0 = 0; i_0 !== lhsVars.size; ++i_0) {
        var key_0 = defDirectVars.get_za3lpa$(i_0);
        var value_0 = lhsVars.get_za3lpa$(i_0);
        map.put_xwzc9p$(key_0, value_0);
      }
      var ifThen = buildIfThen(def);
      if (ifThen.ifSection.clauses.clauses.isEmpty() && ifThen.thenSection.clauses.clauses.size === 1) {
        tmp$_2 = ifThen.thenSection.clauses.clauses.get_za3lpa$(0);
      }
       else {
        tmp$_2 = ifThen;
      }
      var res = tmp$_2;
      return renameVars_0(res, map);
    };
  }
  function replaceIsNodes(node, defs, filter) {
    if (filter === void 0)
      filter = replaceIsNodes$lambda;
    var tmp$;
    var defMap = LinkedHashMap_init();
    tmp$ = defs.iterator();
    while (tmp$.hasNext()) {
      var def = tmp$.next();
      var sig = def.signature;
      if (sig != null) {
        defMap.put_xwzc9p$(sig, def);
      }
    }
    var chalkTransformer = replaceIsNodes$chalkTransformer(filter, defMap);
    return node.transform_nrl0ww$(getCallableRef('chalkTransformer', function (node) {
      return chalkTransformer(node);
    }));
  }
  function toCanonicalForm(def) {
    return new DefinesGroup(def.signature, def.id, def.definesSection, null, new MeansSection(new ClauseListNode(listOf(buildIfThen(def)))), def.aliasSection, def.metaDataSection);
  }
  function buildIfThen(def) {
    var tmp$, tmp$_0;
    return new IfGroup(new IfSection((tmp$_0 = (tmp$ = def.assumingSection) != null ? tmp$.clauses : null) != null ? tmp$_0 : new ClauseListNode(emptyList())), new ThenSection(def.meansSection.clauses));
  }
  function buildIfThen_0(rep) {
    var tmp$, tmp$_0;
    return new IfGroup(new IfSection((tmp$_0 = (tmp$ = rep.assumingSection) != null ? tmp$.clauses : null) != null ? tmp$_0 : new ClauseListNode(emptyList())), new ThenSection(rep.thatSection.clauses));
  }
  function getDefinesDirectVars(def) {
    var tmp$;
    var vars = ArrayList_init();
    tmp$ = def.definesSection.targets.iterator();
    while (tmp$.hasNext()) {
      var target = tmp$.next();
      vars.addAll_brywnq$(getVars_0(target));
    }
    return vars;
  }
  function getDefinesIdVars(def) {
    var vars = ArrayList_init();
    if (Kotlin.isType(def.id.texTalkRoot, ValidationSuccess)) {
      vars.addAll_brywnq$(getVars_1(def.id.texTalkRoot.value));
    }
    return vars;
  }
  function getRepresentsIdVars(rep) {
    var vars = ArrayList_init();
    if (Kotlin.isType(rep.id.texTalkRoot, ValidationSuccess)) {
      vars.addAll_brywnq$(getVars_1(rep.id.texTalkRoot.value));
    }
    return vars;
  }
  function fullExpandOnce$lambda(it) {
    return true;
  }
  function fullExpandOnce$lambda_0(f, f_0) {
    return true;
  }
  function fullExpandOnce$lambda_1(it) {
    return true;
  }
  function fullExpandOnce$lambda_2(it) {
    return true;
  }
  function fullExpandOnce(doc) {
    var tmp$;
    var transformed = separateIsStatements(doc);
    transformed = separateInfixOperatorStatements(transformed);
    transformed = glueCommands(transformed);
    transformed = moveInlineCommandsToIsNode(doc.defines, transformed, fullExpandOnce$lambda, fullExpandOnce$lambda_0);
    transformed = replaceRepresents(transformed, doc.represents, fullExpandOnce$lambda_1);
    return Kotlin.isType(tmp$ = replaceIsNodes(transformed, doc.defines, fullExpandOnce$lambda_2), Document) ? tmp$ : throwCCE();
  }
  function fullExpandComplete(doc, maxSteps) {
    if (maxSteps === void 0)
      maxSteps = 10;
    var snapshots = LinkedHashSet_init();
    var transformed = doc;
    var previousCode = transformed.toCode_eltk6l$(false, 0);
    snapshots.add_11rb$(previousCode);
    for (var i = 0; i < maxSteps; i++) {
      transformed = fullExpandOnce(transformed);
      var code = transformed.toCode_eltk6l$(false, 0);
      if (snapshots.contains_11rb$(code) || equals(previousCode, code)) {
        break;
      }
      previousCode = code;
      snapshots.add_11rb$(previousCode);
    }
    return transformed;
  }
  function separateInfixOperatorStatements$lambda(it) {
    var tmp$, tmp$_0;
    if (Kotlin.isType(it, ClauseListNode)) {
      var newClauses = ArrayList_init();
      tmp$ = it.clauses.iterator();
      while (tmp$.hasNext()) {
        var c = tmp$.next();
        if (Kotlin.isType(c, Statement)) {
          var validation = c.texTalkRoot;
          if (Kotlin.isType(validation, ValidationSuccess)) {
            var root = validation.value;
            tmp$_0 = getExpandedInfixOperators(root).iterator();
            while (tmp$_0.hasNext()) {
              var expanded = tmp$_0.next();
              newClauses.add_11rb$(new Statement(expanded.toCode(), new ValidationSuccess(expanded)));
            }
          }
           else if (Kotlin.isType(validation, ValidationFailure))
            newClauses.add_11rb$(c);
          else
            Kotlin.noWhenBranchMatched();
        }
         else {
          newClauses.add_11rb$(c);
        }
      }
      return new ClauseListNode(newClauses);
    }
     else {
      return it;
    }
  }
  function separateInfixOperatorStatements(phase2Node) {
    return phase2Node.transform_nrl0ww$(separateInfixOperatorStatements$lambda);
  }
  function getSingleInfixOperatorIndex(exp) {
    var tmp$;
    tmp$ = exp.children.size - 1 | 0;
    for (var i = 1; i < tmp$; i++) {
      var prev = exp.children.get_za3lpa$(i - 1 | 0);
      var cur = exp.children.get_za3lpa$(i);
      var next = exp.children.get_za3lpa$(i + 1 | 0);
      if (!isOperator(prev) && Kotlin.isType(cur, Command) && !isOperator(next)) {
        return i;
      }
    }
    return -1;
  }
  function isComma(node) {
    return Kotlin.isType(node, TextTexTalkNode) && equals(node.text, ',');
  }
  function isOperator(node) {
    var tmp$;
    if (!Kotlin.isType(node, TextTexTalkNode)) {
      return false;
    }
    if (isBlank(node.text)) {
      return false;
    }
    tmp$ = iterator(node.text);
    while (tmp$.hasNext()) {
      var c = unboxChar(tmp$.next());
      if (!isOpChar(c)) {
        return false;
      }
    }
    return true;
  }
  function isOpChar(c) {
    return c === 33 || c === 64 || c === 37 || c === 38 || c === 42 || c === 45 || c === 43 || c === 61 || c === 124 || c === 47 || c === 60 || c === 62;
  }
  function getArguments(exp, start, end) {
    var tmp$;
    var result = ArrayList_init();
    var i = start;
    while (i < end) {
      var argChildren = ArrayList_init();
      while (i < end && !isComma(exp.children.get_za3lpa$(i))) {
        argChildren.add_11rb$(exp.children.get_za3lpa$((tmp$ = i, i = tmp$ + 1 | 0, tmp$)));
      }
      if (i < end && isComma(exp.children.get_za3lpa$(i))) {
        i = i + 1 | 0;
      }
      if (argChildren.size === 1) {
        result.add_11rb$(argChildren.get_za3lpa$(0));
      }
       else {
        result.add_11rb$(new ExpressionTexTalkNode(argChildren));
      }
    }
    return result;
  }
  function getExpandedInfixOperators(exp) {
    var tmp$, tmp$_0;
    var opIndex = getSingleInfixOperatorIndex(exp);
    if (opIndex < 0) {
      return listOf(exp);
    }
    var leftArgs = getArguments(exp, 0, opIndex);
    var rightArgs = getArguments(exp, opIndex + 1 | 0, exp.children.size);
    var result = ArrayList_init();
    var op = exp.children.get_za3lpa$(opIndex);
    tmp$ = leftArgs.iterator();
    while (tmp$.hasNext()) {
      var left = tmp$.next();
      tmp$_0 = rightArgs.iterator();
      while (tmp$_0.hasNext()) {
        var right = tmp$_0.next();
        result.add_11rb$(new ExpressionTexTalkNode(listOf_0([left, op, right])));
      }
    }
    return result;
  }
  function Comparator$ObjectLiteral(closure$comparison) {
    this.closure$comparison = closure$comparison;
  }
  Comparator$ObjectLiteral.prototype.compare = function (a, b) {
    return this.closure$comparison(a, b);
  };
  Comparator$ObjectLiteral.$metadata$ = {kind: Kind_CLASS, interfaces: [Comparator]};
  var compareBy$lambda = wrapFunction(function () {
    var compareValues = Kotlin.kotlin.comparisons.compareValues_s00gnj$;
    return function (closure$selector) {
      return function (a, b) {
        var selector = closure$selector;
        return compareValues(selector(a), selector(b));
      };
    };
  });
  function getVars(node) {
    var vars = ArrayList_init();
    getVarsImpl(node, vars);
    return vars;
  }
  function getVars_0(node) {
    var vars = ArrayList_init();
    getVarsImpl_0(node, vars);
    return vars;
  }
  function getVars_1(texTalkNode) {
    var vars = ArrayList_init();
    getVarsImpl_1(texTalkNode, vars, false);
    return vars;
  }
  function renameVars$lambda(closure$map) {
    return function (it) {
      var tmp$;
      if (Kotlin.isType(it, TextTexTalkNode)) {
        return it.copy_buyp7d$(void 0, (tmp$ = closure$map.get_11rb$(it.text)) != null ? tmp$ : it.text);
      }
       else {
        return it;
      }
    };
  }
  function renameVars(texTalkNode, map) {
    return texTalkNode.transform_7szim8$(renameVars$lambda(map));
  }
  function renameVars$chalkTransformer$lambda(it) {
    return it.length;
  }
  function renameVars$chalkTransformer(closure$map) {
    return function (node) {
      var tmp$, tmp$_0, tmp$_1, tmp$_2;
      if (Kotlin.isType(node, Identifier)) {
        return node.copy_61zpoe$((tmp$ = closure$map.get_11rb$(node.name)) != null ? tmp$ : node.name);
      }
      if (Kotlin.isType(node, Statement)) {
        var validation = node.texTalkRoot;
        if (Kotlin.isType(validation, ValidationSuccess)) {
          var exp = Kotlin.isType(tmp$_0 = renameVars(validation.value, closure$map), ExpressionTexTalkNode) ? tmp$_0 : throwCCE();
          return new Statement(exp.toCode(), new ValidationSuccess(exp));
        }
         else if (Kotlin.isType(validation, ValidationFailure))
          tmp$_1 = node;
        else
          tmp$_1 = Kotlin.noWhenBranchMatched();
        return tmp$_1;
      }
       else if (Kotlin.isType(node, Text)) {
        var newText = node.text;
        var keysLongToShort = reversed(sortedWith(toList(closure$map.keys), new Comparator$ObjectLiteral(compareBy$lambda(renameVars$chalkTransformer$lambda))));
        tmp$_2 = keysLongToShort.iterator();
        while (tmp$_2.hasNext()) {
          var key = tmp$_2.next();
          newText = replace(newText, '%' + key, ensureNotNull(closure$map.get_11rb$(key)));
        }
        return new Text(newText);
      }
      return node;
    };
  }
  function renameVars_0(root, map) {
    var chalkTransformer = renameVars$chalkTransformer(map);
    return root.transform_nrl0ww$(getCallableRef('chalkTransformer', function (node) {
      return chalkTransformer(node);
    }));
  }
  function getVarsImpl$lambda(closure$vars) {
    return function (it) {
      getVarsImpl(it, closure$vars);
      return Unit;
    };
  }
  function getVarsImpl(node, vars) {
    if (Kotlin.isType(node, Phase1Token)) {
      vars.add_11rb$(node.text);
    }
     else {
      node.forEach_t0jmzf$(getVarsImpl$lambda(vars));
    }
  }
  function getVarsImpl$lambda_0(closure$vars) {
    return function (it) {
      getVarsImpl_0(it, closure$vars);
      return Unit;
    };
  }
  function getVarsImpl_0(node, vars) {
    if (Kotlin.isType(node, Identifier)) {
      vars.add_11rb$(node.name);
    }
     else if (Kotlin.isType(node, TupleNode)) {
      getVarsImpl(node.tuple, vars);
    }
     else if (Kotlin.isType(node, AggregateNode)) {
      getVarsImpl(node.aggregate, vars);
    }
     else if (Kotlin.isType(node, AbstractionNode)) {
      getVarsImpl(node.abstraction, vars);
    }
     else if (Kotlin.isType(node, AssignmentNode)) {
      vars.add_11rb$(node.assignment.lhs.text);
      getVarsImpl(node.assignment.rhs, vars);
    }
     else {
      node.forEach_ye21ev$(getVarsImpl$lambda_0(vars));
    }
  }
  function getVarsImpl$lambda_1(closure$vars) {
    return function (it) {
      getVarsImpl_1(it, closure$vars, true);
      return Unit;
    };
  }
  function getVarsImpl$lambda_2(closure$vars, closure$inParams) {
    return function (it) {
      getVarsImpl_1(it, closure$vars, closure$inParams);
      return Unit;
    };
  }
  function getVarsImpl_1(texTalkNode, vars, inParams) {
    if (inParams && Kotlin.isType(texTalkNode, TextTexTalkNode)) {
      vars.add_11rb$(texTalkNode.text);
    }
     else if (Kotlin.isType(texTalkNode, ParametersTexTalkNode)) {
      texTalkNode.forEach_j2ps96$(getVarsImpl$lambda_1(vars));
    }
     else {
      texTalkNode.forEach_j2ps96$(getVarsImpl$lambda_2(vars, inParams));
    }
  }
  var package$mathlingua = _.mathlingua || (_.mathlingua = {});
  var package$common = package$mathlingua.common || (package$mathlingua.common = {});
  package$common.MathLingua = MathLingua;
  package$common.ParseError = ParseError;
  package$common.Validation = Validation;
  package$common.ValidationSuccess = ValidationSuccess;
  package$common.ValidationFailure = ValidationFailure;
  package$common.Stack = Stack;
  package$common.Queue = Queue;
  var package$chalktalk = package$common.chalktalk || (package$common.chalktalk = {});
  var package$phase1 = package$chalktalk.phase1 || (package$chalktalk.phase1 = {});
  package$phase1.ChalkTalkLexer = ChalkTalkLexer;
  package$phase1.newChalkTalkLexer_61zpoe$ = newChalkTalkLexer;
  package$phase1.ChalkTalkParser = ChalkTalkParser;
  package$phase1.ChalkTalkParseResult = ChalkTalkParseResult;
  package$phase1.newChalkTalkParser = newChalkTalkParser;
  var package$ast = package$phase1.ast || (package$phase1.ast = {});
  package$ast.Phase1Node = Phase1Node;
  package$ast.Root = Root;
  package$ast.Argument = Argument;
  package$ast.Section = Section;
  package$ast.buildIndent_fzusl$ = buildIndent;
  package$ast.getIndent_za3lpa$ = getIndent;
  package$ast.getRow_baevx2$ = getRow;
  package$ast.getColumn_baevx2$ = getColumn;
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
  package$ast.Phase1Target = Phase1Target;
  package$ast.TupleItem = TupleItem;
  package$ast.AssignmentRhs = AssignmentRhs;
  package$ast.Phase1Token = Phase1Token;
  package$ast.Mapping = Mapping;
  package$ast.Group = Group;
  package$ast.Assignment = Assignment;
  package$ast.Tuple = Tuple;
  package$ast.Abstraction = Abstraction;
  package$ast.Aggregate = Aggregate;
  var package$phase2 = package$chalktalk.phase2 || (package$chalktalk.phase2 = {});
  package$phase2.AliasSection = AliasSection;
  package$phase2.validateAliasSection_3fjnpj$ = validateAliasSection;
  package$phase2.indentedString_qta3xh$ = indentedString;
  package$phase2.Clause = Clause;
  package$phase2.validateClause_baevx2$ = validateClause;
  package$phase2.Target = Target;
  package$phase2.AbstractionNode = AbstractionNode;
  package$phase2.isAbstraction_baevx2$ = isAbstraction;
  package$phase2.validateAbstractionNode_baevx2$ = validateAbstractionNode;
  package$phase2.AggregateNode = AggregateNode;
  package$phase2.isAggregate_baevx2$ = isAggregate;
  package$phase2.validateAggregateNode_baevx2$ = validateAggregateNode;
  package$phase2.TupleNode = TupleNode;
  package$phase2.isTuple_baevx2$ = isTuple;
  package$phase2.validateTupleNode_baevx2$ = validateTupleNode;
  package$phase2.AssignmentNode = AssignmentNode;
  package$phase2.isAssignment_baevx2$ = isAssignment;
  package$phase2.validateAssignmentNode_baevx2$ = validateAssignmentNode;
  package$phase2.MappingNode = MappingNode;
  package$phase2.isMapping_baevx2$ = isMapping;
  package$phase2.validateMappingNode_baevx2$ = validateMappingNode;
  package$phase2.Identifier = Identifier;
  package$phase2.isIdentifier_baevx2$ = isIdentifier;
  package$phase2.validateIdentifier_baevx2$ = validateIdentifier;
  package$phase2.Statement = Statement;
  package$phase2.isStatement_baevx2$ = isStatement;
  package$phase2.validateStatement_baevx2$ = validateStatement;
  package$phase2.Text = Text;
  package$phase2.isText_baevx2$ = isText;
  package$phase2.validateText_baevx2$ = validateText;
  package$phase2.ExistsGroup = ExistsGroup;
  package$phase2.isExistsGroup_baevx2$ = isExistsGroup;
  package$phase2.validateExistsGroup_baevx2$ = validateExistsGroup;
  package$phase2.IfGroup = IfGroup;
  package$phase2.isIfGroup_baevx2$ = isIfGroup;
  package$phase2.validateIfGroup_baevx2$ = validateIfGroup;
  package$phase2.IffGroup = IffGroup;
  package$phase2.isIffGroup_baevx2$ = isIffGroup;
  package$phase2.validateIffGroup_baevx2$ = validateIffGroup;
  package$phase2.ForGroup = ForGroup;
  package$phase2.isForGroup_baevx2$ = isForGroup;
  package$phase2.validateForGroup_baevx2$ = validateForGroup;
  package$phase2.NotGroup = NotGroup;
  package$phase2.isNotGroup_baevx2$ = isNotGroup;
  package$phase2.validateNotGroup_baevx2$ = validateNotGroup;
  package$phase2.OrGroup = OrGroup;
  package$phase2.isOrGroup_baevx2$ = isOrGroup;
  package$phase2.validateOrGroup_baevx2$ = validateOrGroup;
  package$phase2.firstSectionMatchesName_ipjtm0$ = firstSectionMatchesName;
  package$phase2.validateSingleSectionGroup_ftzhui$ = validateSingleSectionGroup;
  package$phase2.toCode_h1xtml$ = toCode;
  package$phase2.toCode_tsx3j7$ = toCode_0;
  package$phase2.ClauseListNode = ClauseListNode;
  package$phase2.ClauseListSection = ClauseListSection;
  package$phase2.validateClauseList_a0jl34$ = validateClauseList;
  package$phase2.Phase2Node = Phase2Node;
  package$phase2.Document = Document;
  package$phase2.validateDocument_baevx2$ = validateDocument;
  package$phase2.SourceGroup = SourceGroup;
  package$phase2.isSourceGroup_baevx2$ = isSourceGroup;
  package$phase2.validateSourceGroup_hzi7mn$ = validateSourceGroup;
  package$phase2.DefinesGroup = DefinesGroup;
  package$phase2.isDefinesGroup_baevx2$ = isDefinesGroup;
  package$phase2.validateDefinesGroup_hzi7mn$ = validateDefinesGroup;
  package$phase2.RepresentsGroup = RepresentsGroup;
  package$phase2.isRepresentsGroup_baevx2$ = isRepresentsGroup;
  package$phase2.validateRepresentsGroup_hzi7mn$ = validateRepresentsGroup;
  package$phase2.ResultGroup = ResultGroup;
  package$phase2.isResultGroup_baevx2$ = isResultGroup;
  package$phase2.validateResultGroup_hzi7mn$ = validateResultGroup;
  package$phase2.AxiomGroup = AxiomGroup;
  package$phase2.isAxiomGroup_baevx2$ = isAxiomGroup;
  package$phase2.validateAxiomGroup_hzi7mn$ = validateAxiomGroup;
  package$phase2.ConjectureGroup = ConjectureGroup;
  package$phase2.isConjectureGroup_baevx2$ = isConjectureGroup;
  package$phase2.validateConjectureGroup_hzi7mn$ = validateConjectureGroup;
  package$phase2.toCode_2d2cwo$ = toCode_1;
  package$phase2.validateResultLikeGroup_g6v280$ = validateResultLikeGroup;
  package$phase2.validateDefinesLikeGroup_yceizu$ = validateDefinesLikeGroup;
  package$phase2.MetaDataSection = MetaDataSection;
  package$phase2.validateMetaDataSection_3fjnpj$ = validateMetaDataSection;
  package$phase2.AssumingSection = AssumingSection;
  package$phase2.validateAssumingSection_baevx2$ = validateAssumingSection;
  package$phase2.DefinesSection = DefinesSection;
  package$phase2.validateDefinesSection_baevx2$ = validateDefinesSection;
  package$phase2.RefinesSection = RefinesSection;
  package$phase2.validateRefinesSection_baevx2$ = validateRefinesSection;
  package$phase2.RepresentsSection = RepresentsSection;
  package$phase2.validateRepresentsSection_baevx2$ = validateRepresentsSection;
  package$phase2.ExistsSection = ExistsSection;
  package$phase2.validateExistsSection_baevx2$ = validateExistsSection;
  package$phase2.ForSection = ForSection;
  package$phase2.validateForSection_baevx2$ = validateForSection;
  package$phase2.MeansSection = MeansSection;
  package$phase2.validateMeansSection_baevx2$ = validateMeansSection;
  package$phase2.ResultSection = ResultSection;
  package$phase2.validateResultSection_baevx2$ = validateResultSection;
  package$phase2.AxiomSection = AxiomSection;
  package$phase2.validateAxiomSection_baevx2$ = validateAxiomSection;
  package$phase2.ConjectureSection = ConjectureSection;
  package$phase2.validateConjectureSection_baevx2$ = validateConjectureSection;
  package$phase2.SuchThatSection = SuchThatSection;
  package$phase2.validateSuchThatSection_baevx2$ = validateSuchThatSection;
  package$phase2.ThatSection = ThatSection;
  package$phase2.validateThatSection_baevx2$ = validateThatSection;
  package$phase2.IfSection = IfSection;
  package$phase2.validateIfSection_baevx2$ = validateIfSection;
  package$phase2.IffSection = IffSection;
  package$phase2.validateIffSection_baevx2$ = validateIffSection;
  package$phase2.ThenSection = ThenSection;
  package$phase2.validateThenSection_baevx2$ = validateThenSection;
  package$phase2.WhereSection = WhereSection;
  package$phase2.validateWhereSection_baevx2$ = validateWhereSection;
  package$phase2.NotSection = NotSection;
  package$phase2.validateNotSection_baevx2$ = validateNotSection;
  package$phase2.OrSection = OrSection;
  package$phase2.validateOrSection_baevx2$ = validateOrSection;
  package$phase2.identifySections_b3nzct$ = identifySections;
  package$phase2.SourceSection = SourceSection;
  package$phase2.validateSourceSection_3fjnpj$ = validateSourceSection;
  package$phase2.TargetListSection = TargetListSection;
  package$phase2.validateTargetList_oryd0v$ = validateTargetList;
  Object.defineProperty(TexTalkNodeType, 'Token', {
    get: TexTalkNodeType$Token_getInstance
  });
  Object.defineProperty(TexTalkNodeType, 'Identifier', {
    get: TexTalkNodeType$Identifier_getInstance
  });
  Object.defineProperty(TexTalkNodeType, 'Operator', {
    get: TexTalkNodeType$Operator_getInstance
  });
  Object.defineProperty(TexTalkNodeType, 'ParenGroup', {
    get: TexTalkNodeType$ParenGroup_getInstance
  });
  Object.defineProperty(TexTalkNodeType, 'SquareGroup', {
    get: TexTalkNodeType$SquareGroup_getInstance
  });
  Object.defineProperty(TexTalkNodeType, 'CurlyGroup', {
    get: TexTalkNodeType$CurlyGroup_getInstance
  });
  Object.defineProperty(TexTalkNodeType, 'NamedGroup', {
    get: TexTalkNodeType$NamedGroup_getInstance
  });
  Object.defineProperty(TexTalkNodeType, 'Command', {
    get: TexTalkNodeType$Command_getInstance
  });
  Object.defineProperty(TexTalkNodeType, 'CommandPart', {
    get: TexTalkNodeType$CommandPart_getInstance
  });
  Object.defineProperty(TexTalkNodeType, 'Expression', {
    get: TexTalkNodeType$Expression_getInstance
  });
  Object.defineProperty(TexTalkNodeType, 'SubSup', {
    get: TexTalkNodeType$SubSup_getInstance
  });
  Object.defineProperty(TexTalkNodeType, 'Parameters', {
    get: TexTalkNodeType$Parameters_getInstance
  });
  Object.defineProperty(TexTalkNodeType, 'Comma', {
    get: TexTalkNodeType$Comma_getInstance
  });
  Object.defineProperty(TexTalkNodeType, 'Is', {
    get: TexTalkNodeType$Is_getInstance
  });
  Object.defineProperty(TexTalkNodeType, 'ColonEquals', {
    get: TexTalkNodeType$ColonEquals_getInstance
  });
  var package$textalk = package$common.textalk || (package$common.textalk = {});
  package$textalk.TexTalkNodeType = TexTalkNodeType;
  package$textalk.TexTalkNode = TexTalkNode;
  package$textalk.IsTexTalkNode = IsTexTalkNode;
  package$textalk.ColonEqualsTexTalkNode = ColonEqualsTexTalkNode;
  package$textalk.CommandPart = CommandPart;
  package$textalk.Command = Command;
  package$textalk.ExpressionTexTalkNode = ExpressionTexTalkNode;
  package$textalk.ParametersTexTalkNode = ParametersTexTalkNode;
  package$textalk.GroupTexTalkNode = GroupTexTalkNode;
  package$textalk.NamedGroupTexTalkNode = NamedGroupTexTalkNode;
  package$textalk.SubSupTexTalkNode = SubSupTexTalkNode;
  package$textalk.TextTexTalkNode = TextTexTalkNode;
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
  package$textalk.getAncestry_3b8392$ = getAncestry;
  package$textalk.TexTalkLexer = TexTalkLexer;
  package$textalk.newTexTalkLexer_61zpoe$ = newTexTalkLexer;
  package$textalk.TexTalkParser = TexTalkParser;
  package$textalk.TexTalkParseResult = TexTalkParseResult;
  package$textalk.newTexTalkParser = newTexTalkParser;
  package$textalk.TexTalkParserImpl = TexTalkParserImpl;
  var package$transform = package$common.transform || (package$common.transform = {});
  package$transform.findCommands_w5twsl$ = findCommands;
  package$transform.replaceSignatures_eg7yu9$ = replaceSignatures;
  package$transform.replaceCommands_nd4h0u$ = replaceCommands;
  package$transform.replaceCommands_3f9q6x$ = replaceCommands_0;
  package$transform.separateIsStatements_mu0sga$ = separateIsStatements;
  package$transform.glueCommands_mu0sga$ = glueCommands;
  package$transform.glueCommands_kty9eo$ = glueCommands_0;
  package$transform.getSignature_c9m4n2$ = getSignature;
  package$transform.findAllStatementSignatures_c9m4n2$ = findAllStatementSignatures;
  package$transform.getMergedCommandSignature_p9tldv$ = getMergedCommandSignature;
  package$transform.getCommandSignature_mwzhn3$ = getCommandSignature;
  package$transform.locateAllSignatures_mu0sga$ = locateAllSignatures;
  package$transform.moveInlineCommandsToIsNode_wly2kn$ = moveInlineCommandsToIsNode;
  package$transform.moveStatementInlineCommandsToIsNode_i4q3sa$ = moveStatementInlineCommandsToIsNode;
  package$transform.replaceRepresents_pkrmav$ = replaceRepresents;
  package$transform.replaceIsNodes_689pkc$ = replaceIsNodes;
  package$transform.toCanonicalForm_gdejq0$ = toCanonicalForm;
  package$transform.buildIfThen_gdejq0$ = buildIfThen;
  package$transform.buildIfThen_xz2koz$ = buildIfThen_0;
  package$transform.getDefinesDirectVars_gdejq0$ = getDefinesDirectVars;
  package$transform.getDefinesIdVars_gdejq0$ = getDefinesIdVars;
  package$transform.getRepresentsIdVars_xz2koz$ = getRepresentsIdVars;
  package$transform.fullExpandOnce_8vvjcc$ = fullExpandOnce;
  package$transform.fullExpandComplete_z4xgv2$ = fullExpandComplete;
  package$transform.separateInfixOperatorStatements_mu0sga$ = separateInfixOperatorStatements;
  package$transform.getVars_baevx2$ = getVars;
  package$transform.getVars_mu0sga$ = getVars_0;
  package$transform.getVars_w5twsl$ = getVars_1;
  package$transform.renameVars_m39xoo$ = renameVars;
  package$transform.renameVars_gh6ac9$ = renameVars_0;
  INVALID = new Phase1Token('INVALID', ChalkTalkTokenType$Invalid_getInstance(), -1, -1);
  CLAUSE_VALIDATORS = listOf_0([new ValidationPair(getCallableRef('isAbstraction', function (node) {
    return isAbstraction(node);
  }), getCallableRef('validateAbstractionNode', function (node) {
    return validateAbstractionNode(node);
  })), new ValidationPair(getCallableRef('isAggregate', function (node) {
    return isAggregate(node);
  }), getCallableRef('validateAggregateNode', function (node) {
    return validateAggregateNode(node);
  })), new ValidationPair(getCallableRef('isTuple', function (node) {
    return isTuple(node);
  }), getCallableRef('validateTupleNode', function (node) {
    return validateTupleNode(node);
  })), new ValidationPair(getCallableRef('isAssignment', function (node) {
    return isAssignment(node);
  }), getCallableRef('validateAssignmentNode', function (node) {
    return validateAssignmentNode(node);
  })), new ValidationPair(getCallableRef('isIdentifier', function (node) {
    return isIdentifier(node);
  }), getCallableRef('validateIdentifier', function (rawNode) {
    return validateIdentifier(rawNode);
  })), new ValidationPair(getCallableRef('isStatement', function (node) {
    return isStatement(node);
  }), getCallableRef('validateStatement', function (rawNode) {
    return validateStatement(rawNode);
  })), new ValidationPair(getCallableRef('isText', function (node) {
    return isText(node);
  }), getCallableRef('validateText', function (rawNode) {
    return validateText(rawNode);
  })), new ValidationPair(getCallableRef('isForGroup', function (node) {
    return isForGroup(node);
  }), getCallableRef('validateForGroup', function (rawNode) {
    return validateForGroup(rawNode);
  })), new ValidationPair(getCallableRef('isExistsGroup', function (node) {
    return isExistsGroup(node);
  }), getCallableRef('validateExistsGroup', function (node) {
    return validateExistsGroup(node);
  })), new ValidationPair(getCallableRef('isNotGroup', function (node) {
    return isNotGroup(node);
  }), getCallableRef('validateNotGroup', function (node) {
    return validateNotGroup(node);
  })), new ValidationPair(getCallableRef('isOrGroup', function (node) {
    return isOrGroup(node);
  }), getCallableRef('validateOrGroup', function (node) {
    return validateOrGroup(node);
  })), new ValidationPair(getCallableRef('isIfGroup', function (node) {
    return isIfGroup(node);
  }), getCallableRef('validateIfGroup', function (node) {
    return validateIfGroup(node);
  })), new ValidationPair(getCallableRef('isIffGroup', function (node) {
    return isIffGroup(node);
  }), getCallableRef('validateIffGroup', function (node) {
    return validateIffGroup(node);
  }))]);
  INVALID_0 = new TexTalkToken('INVALID', TexTalkTokenType$Invalid_getInstance(), -1, -1);
  Kotlin.defineModule('bundle', _);
  return _;
}(typeof bundle === 'undefined' ? {} : bundle, kotlin);
