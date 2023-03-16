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

type ExpAliasInfoType interface {
	ExpAliasInfoType()
}

func (InfixExpAliasInfo) ExpAliasInfoType()          {}
func (PrefixExpAliasInfo) ExpAliasInfoType()         {}
func (PostfixExpAliasInfo) ExpAliasInfoType()        {}
func (FunctionExpAliasInfo) ExpAliasInfoType()       {}
func (CommandExpAliasInfo) ExpAliasInfoType()        {}
func (MemberNameExpAliasInfo) ExpAliasInfoType()     {}
func (MemberFunctionExpAliasInfo) ExpAliasInfoType() {}
func (MemberInfixExpAliasInfo) ExpAliasInfoType()    {}
func (MemberPrefixExpAliasInfo) ExpAliasInfoType()   {}
func (MemberPostfixExpAliasInfo) ExpAliasInfoType()  {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type InfixExpAliasInfo struct {
	Lhs InfixStaticPattern
	Rhs *ExpressionType
}

type PrefixExpAliasInfo struct {
	Lhs PrefixStaticPattern
	Rhs *ExpressionType
}

type PostfixExpAliasInfo struct {
	Lhs PostfixStaticPattern
	Rhs *ExpressionType
}

type FunctionExpAliasInfo struct {
	Lhs FunctionStaticPattern
	Rhs *ExpressionType
}

type CommandExpAliasInfo struct {
	Lsh CommandStaticPattern
	Rhs *ExpressionType
}

type MemberNameExpAliasInfo struct {
	Lsh MemberNameStaticPattern
	Rhs *ExpressionType
}

type MemberFunctionExpAliasInfo struct {
	Lsh MemberFunctionStaticPattern
	Rhs *ExpressionType
}

type MemberInfixExpAliasInfo struct {
	Lsh MemberInfixStaticPattern
	Rhs *ExpressionType
}

type MemberPrefixExpAliasInfo struct {
	Lhs MemberPrefixStaticPattern
	Rhs *ExpressionType
}

type MemberPostfixExpAliasInfo struct {
	Lsh MemberPostfixStaticPattern
	Rhs *ExpressionType
}
