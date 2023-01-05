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

type Context interface {
	HasIdentifier(name string) bool
	GetTypeInfo(name string) (TypeInfo, error)
	GetLocalIdentifiers() []string
	AddIdentifier(name string, info TypeInfo) error
	AddIdentifierConstraint(name string, command ast.CommandExpression) error
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

func NewWritableContext(parent Context) Context {
	return &context{
		writable:  true,
		parent:    parent,
		classes:   make(map[string]IdentifierClass),
		typeInfos: make(map[string]TypeInfo),
	}
}

func NewReadableContext(parent Context) Context {
	return &context{
		writable:  false,
		parent:    parent,
		classes:   make(map[string]IdentifierClass),
		typeInfos: make(map[string]TypeInfo),
	}
}

/////////////////////////////////////////////////////////////

type context struct {
	writable  bool
	parent    Context
	classes   map[string]IdentifierClass
	typeInfos map[string]TypeInfo
}

func (r *context) AddIdentifier(name string, info TypeInfo) error {
	if !r.writable {
		if r.parent != nil {
			return r.parent.AddIdentifier(name, info)
		} else {
			return fmt.Errorf("cannot add name %s to a readonly context", name)
		}
	}

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

func (r *context) AddIdentifierConstraint(name string, command ast.CommandExpression) error {
	if !r.writable {
		if r.parent != nil {
			return r.parent.AddIdentifierConstraint(name, command)
		} else {
			return fmt.Errorf("cannot constraint to name %s in a readonly context", name)
		}
	}

	info, err := r.GetTypeInfo(name)
	if err != nil {
		// GetTypeInfo already tries to get the TypeInfo from the parent
		// if it doesn't exist in the current context
		return err
	}
	info.AddConstraint(command)
	return nil
}

func (r *context) MarkEquivalent(name1 string, name2 string) error {
	if !r.writable {
		if r.parent != nil {
			return r.parent.MarkEquivalent(name1, name2)
		} else {
			return fmt.Errorf("cannot mark %s := %s in a readonly context", name1, name2)
		}
	}

	cls, ok := r.classes[name1]
	if !ok {
		if r.parent != nil {
			return r.parent.MarkEquivalent(name1, name2)
		} else {
			return fmt.Errorf("the name %s does not exist in this context", name1)
		}
	}
	cls.AddIdentifier(name2)
	return nil
}

func (r *context) HasIdentifier(name string) bool {
	for i := range r.classes {
		if r.classes[i].HasIdentifier(name) {
			return true
		}
	}
	if r.parent != nil {
		return r.parent.HasIdentifier(name)
	} else {
		return false
	}
}

func (r *context) GetTypeInfo(name string) (TypeInfo, error) {
	if info, ok := r.typeInfos[name]; ok {
		return info, nil
	}
	if r.parent != nil {
		return r.GetTypeInfo(name)
	} else {
		return nil, fmt.Errorf("unknown name %s", name)
	}
}

func (r *context) GetLocalIdentifiers() []string {
	result := make([]string, 0)
	for i := range r.classes {
		cls := r.classes[i]
		result = append(result, cls.GetAllIdentifiers()...)
	}
	return result
}
