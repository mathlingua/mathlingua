/*
 * Copyright 2023 Dominic Kramer
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package ast

type FormulationNodeKind interface {
	MlgNodeKind
	FormulationNodeType()
	Start() Position
	GetFormulationMetaData() *FormulationMetaData
	ToCode(fn func(node MlgNodeKind) (string, bool)) string
}

type FormulationMetaData struct {
	Original FormulationNodeKind
}

func (*NameForm) FormulationNodeType()                               {}
func (*FunctionForm) FormulationNodeType()                           {}
func (*TupleForm) FormulationNodeType()                              {}
func (*ConditionalSetForm) FormulationNodeType()                     {}
func (*ConditionalSetIdForm) FormulationNodeType()                   {}
func (*FunctionCallExpression) FormulationNodeType()                 {}
func (*TupleExpression) FormulationNodeType()                        {}
func (*ConditionalSetExpression) FormulationNodeType()               {}
func (*CommandExpression) FormulationNodeType()                      {}
func (*PrefixOperatorCallExpression) FormulationNodeType()           {}
func (*PostfixOperatorCallExpression) FormulationNodeType()          {}
func (*InfixOperatorCallExpression) FormulationNodeType()            {}
func (*IsExpression) FormulationNodeType()                           {}
func (*ExtendsExpression) FormulationNodeType()                      {}
func (*AsExpression) FormulationNodeType()                           {}
func (*OrdinalCallExpression) FormulationNodeType()                  {}
func (*ChainExpression) FormulationNodeType()                        {}
func (*Signature) FormulationNodeType()                              {}
func (*MetaKinds) FormulationNodeType()                              {}
func (*StructuralColonEqualsForm) FormulationNodeType()              {}
func (*ExpressionColonEqualsItem) FormulationNodeType()              {}
func (*ExpressionColonArrowItem) FormulationNodeType()               {}
func (*ExpressionColonDashArrowItem) FormulationNodeType()           {}
func (*EnclosedNonCommandOperatorTarget) FormulationNodeType()       {}
func (*NonEnclosedNonCommandOperatorTarget) FormulationNodeType()    {}
func (*CommandOperatorTarget) FormulationNodeType()                  {}
func (*CommandId) FormulationNodeType()                              {}
func (*PrefixOperatorId) FormulationNodeType()                       {}
func (*PostfixOperatorId) FormulationNodeType()                      {}
func (*InfixOperatorId) FormulationNodeType()                        {}
func (*InfixCommandOperatorId) FormulationNodeType()                 {}
func (*PseudoTokenNode) FormulationNodeType()                        {}
func (*PseudoExpression) FormulationNodeType()                       {}
func (*MultiplexedInfixOperatorCallExpression) FormulationNodeType() {}
func (*InfixOperatorForm) FormulationNodeType()                      {}
func (*PrefixOperatorForm) FormulationNodeType()                     {}
func (*PostfixOperatorForm) FormulationNodeType()                    {}
func (*FunctionLiteralExpression) FormulationNodeType()              {}
func (*FunctionLiteralForm) FormulationNodeType()                    {}
