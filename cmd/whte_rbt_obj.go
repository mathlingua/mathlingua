package cmd

import (
	"fmt"
	"time"

	"github.com/spf13/cobra"
)

var whteRbtObjCommand = &cobra.Command{
	Use:    "whte_rbt.obj",
	Hidden: true,
	Run: func(cmd *cobra.Command, args []string) {
		whteRbtObj()
	},
}

func init() {
	rootCmd.AddCommand(whteRbtObjCommand)
}

func whteRbtObj() {
	fmt.Println("Jurassic Park, System Security Interface")
	time.Sleep(700 * time.Millisecond)
	fmt.Println("Version 4.0.5, Alpha E")
	time.Sleep(700 * time.Millisecond)
	fmt.Println("Ready...")
	time.Sleep(700 * time.Millisecond)
	fmt.Print("> ")

	accessSecurity := "access security"
	for _, char := range accessSecurity {
		time.Sleep(100 * time.Millisecond)
		fmt.Print(string(char))
	}
	fmt.Println()
	time.Sleep(700 * time.Millisecond)
	fmt.Println("access: PERMISSION DENIED.")
	fmt.Print("> ")

	accessSecurityGrid := "access security grid"
	for _, char := range accessSecurityGrid {
		time.Sleep(100 * time.Millisecond)
		fmt.Print(string(char))
	}
	fmt.Println()
	time.Sleep(700 * time.Millisecond)
	fmt.Println("access: PERMISSION DENIED.")
	fmt.Print("> ")

	accessMainSecurityGrid := "access main security grid"
	for _, char := range accessMainSecurityGrid {
		time.Sleep(100 * time.Millisecond)
		fmt.Print(string(char))
	}
	fmt.Println()
	time.Sleep(1000 * time.Millisecond)
	fmt.Print("access: PERMISSION DENIED.")
	time.Sleep(700 * time.Millisecond)
	fmt.Println("...and....")
	time.Sleep(1000 * time.Millisecond)

	for {
		fmt.Println("YOU DIDN'T SAY THE MAGIC WORD!")
		time.Sleep(50 * time.Millisecond)
	}
}
