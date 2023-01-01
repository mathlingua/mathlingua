/*
 * Copyright 2022 Dominic Kramer
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

package backend

import (
	"fmt"
	"mathlingua/internal/ast"
)

type ReadableContext interface {
	HasIdentifier(name string) bool
	GetTypeInfo(name string) (TypeInfo, error)
	GetAllIdentifiers() []string
}

type WriteableContext interface {
	ReadableContext
	AddIdentifier(name string, info TypeInfo) error
	AddIdentifierConstraint(name string, command ast.CommandExpression) error
	AddIdentifierAtConstraint(name string, command ast.CommandAtExpression) error
	// specifies `name1 := name2`
	MarkEquivalent(name1 string, name2 string) error
}

func GetIdentifier(node ast.MlgNodeType) string {
	switch node := node.(type) {
	case *ast.FunctionCallExpression:
		return node.Target.Debug() + "()"
	case *ast.FunctionExpressionForm:
		return node.Target.Debug() + "()"
	case *ast.FunctionForm:
		return node.Target.Debug() + "()"
	default:
		return ast.Debug(node)
	}
}

func NewWritableContext() WriteableContext {
	return &writableContext{}
}

func NewReadableContext() ReadableContext {
	return &writableContext{}
}

/////////////////////////////////////////////////////////////

type writableContext struct {
	classes   map[string]IdentifierClass
	typeInfos map[string]TypeInfo
}

func (r *writableContext) AddIdentifier(name string, info TypeInfo) error {
	_, ok := r.classes[name]
	if ok {
		return fmt.Errorf("the name %s already exists in this context", name)
	}
	cls := NewIdentifierClass()
	cls.AddIdentifier(name)
	r.classes[name] = cls
	r.typeInfos[name] = info
	return nil
}

func (r *writableContext) AddIdentifierConstraint(name string, command ast.CommandExpression) error {
	info, err := r.GetTypeInfo(name)
	if err != nil {
		return err
	}
	info.AddConstraint(command)
	return nil
}

func (r *writableContext) AddIdentifierAtConstraint(name string, command ast.CommandAtExpression) error {
	info, err := r.GetTypeInfo(name)
	if err != nil {
		return err
	}
	info.AddAtConstraint(command)
	return nil
}

func (r *writableContext) MarkEquivalent(name1 string, name2 string) error {
	cls, ok := r.classes[name1]
	if !ok {
		return fmt.Errorf("the name %s does not exist in this context", name1)
	}
	cls.AddIdentifier(name2)
	return nil
}

func (r *writableContext) HasIdentifier(name string) bool {
	for i := range r.classes {
		if r.classes[i].HasIdentifier(name) {
			return true
		}
	}
	return false
}

func (r *writableContext) GetTypeInfo(name string) (TypeInfo, error) {
	if info, ok := r.typeInfos[name]; ok {
		return info, nil
	}
	return nil, fmt.Errorf("unknown name %s", name)
}

func (r *writableContext) GetAllIdentifiers() []string {
	result := make([]string, 0)
	for i := range r.classes {
		cls := r.classes[i]
		result = append(result, cls.GetAllIdentifiers()...)
	}
	return result
}
