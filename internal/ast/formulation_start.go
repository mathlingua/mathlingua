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

func (n *NameForm) Start() Position                               { return n.CommonMetaData.Start }
func (n *FunctionForm) Start() Position                           { return n.CommonMetaData.Start }
func (n *ExpressionForm) Start() Position                         { return n.CommonMetaData.Start }
func (n *TupleForm) Start() Position                              { return n.CommonMetaData.Start }
func (n *ConditionalSetForm) Start() Position                     { return n.CommonMetaData.Start }
func (n *ConditionalSetIdForm) Start() Position                   { return n.CommonMetaData.Start }
func (n *FunctionCallExpression) Start() Position                 { return n.CommonMetaData.Start }
func (n *TupleExpression) Start() Position                        { return n.CommonMetaData.Start }
func (n *LabeledGrouping) Start() Position                        { return n.CommonMetaData.Start }
func (n *ConditionalSetExpression) Start() Position               { return n.CommonMetaData.Start }
func (n *CommandExpression) Start() Position                      { return n.CommonMetaData.Start }
func (n *PrefixOperatorCallExpression) Start() Position           { return n.CommonMetaData.Start }
func (n *PostfixOperatorCallExpression) Start() Position          { return n.CommonMetaData.Start }
func (n *InfixOperatorCallExpression) Start() Position            { return n.CommonMetaData.Start }
func (n *IsExpression) Start() Position                           { return n.CommonMetaData.Start }
func (n *AsExpression) Start() Position                           { return n.CommonMetaData.Start }
func (n *OrdinalCallExpression) Start() Position                  { return n.CommonMetaData.Start }
func (n *ChainExpression) Start() Position                        { return n.CommonMetaData.Start }
func (n *Signature) Start() Position                              { return n.CommonMetaData.Start }
func (n *StructuralColonEqualsForm) Start() Position              { return n.CommonMetaData.Start }
func (n *StructuralColonEqualsColonForm) Start() Position         { return n.CommonMetaData.Start }
func (n *ExpressionColonEqualsItem) Start() Position              { return n.CommonMetaData.Start }
func (n *ExpressionColonArrowItem) Start() Position               { return n.CommonMetaData.Start }
func (n *ExpressionColonDashArrowItem) Start() Position           { return n.CommonMetaData.Start }
func (n *EnclosedNonCommandOperatorTarget) Start() Position       { return n.CommonMetaData.Start }
func (n *NonEnclosedNonCommandOperatorTarget) Start() Position    { return n.CommonMetaData.Start }
func (n *InfixCommandExpression) Start() Position                 { return n.CommonMetaData.Start }
func (n *CommandId) Start() Position                              { return n.CommonMetaData.Start }
func (n *PrefixOperatorId) Start() Position                       { return n.CommonMetaData.Start }
func (n *PostfixOperatorId) Start() Position                      { return n.CommonMetaData.Start }
func (n *InfixOperatorId) Start() Position                        { return n.CommonMetaData.Start }
func (n *InfixCommandOperatorId) Start() Position                 { return n.CommonMetaData.Start }
func (n *PseudoTokenNode) Start() Position                        { return n.CommonMetaData.Start }
func (n *PseudoExpression) Start() Position                       { return n.CommonMetaData.Start }
func (n *MultiplexedInfixOperatorCallExpression) Start() Position { return n.CommonMetaData.Start }
func (n *InfixOperatorForm) Start() Position                      { return n.CommonMetaData.Start }
func (n *PrefixOperatorForm) Start() Position                     { return n.CommonMetaData.Start }
func (n *PostfixOperatorForm) Start() Position                    { return n.CommonMetaData.Start }
func (n *FunctionLiteralExpression) Start() Position              { return n.CommonMetaData.Start }
func (n *FunctionLiteralForm) Start() Position                    { return n.CommonMetaData.Start }
func (n *DefinitionBuiltinExpression) Start() Position            { return n.CommonMetaData.Start }
func (n *MapToElseBuiltinExpression) Start() Position             { return n.CommonMetaData.Start }
func (n *CommandTypeForm) Start() Position                        { return n.CommonMetaData.Start }
func (n *InfixCommandTypeForm) Start() Position                   { return n.CommonMetaData.Start }
func (n *AbstractBuiltinExpression) Start() Position              { return n.CommonMetaData.Start }
func (n *SpecificationBuiltinExpression) Start() Position         { return n.CommonMetaData.Start }
func (n *StatementBuiltinExpression) Start() Position             { return n.CommonMetaData.Start }
func (n *ExpressionBuiltinExpression) Start() Position            { return n.CommonMetaData.Start }
func (n *TypeBuiltinExpression) Start() Position                  { return n.CommonMetaData.Start }
