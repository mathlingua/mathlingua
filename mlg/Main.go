package main

import (
	"bufio"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"runtime"
	"strings"
)

const MATHLINGUA_VERSION = "0.11.0"

type Release struct {
	mlgUrl string
	jarUrl string
}

func getNewMlgPath() (string, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return "", err
	}
	return path.Join(cwd, ".bin", "mlg.new"), nil
}

func runMathLingua(args []string) (int, error) {
	cmdArgs := []string{
		"-jar",
		path.Join(".bin", "mathlingua.jar"),
	}
	cmdArgs = append(cmdArgs, args...)
	return runCommand("java", cmdArgs)
}

func runCommand(program string, args []string) (int, error) {
	cmd := exec.Command(program, args...)

	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return 1, err
	}

	stderr, err := cmd.StderrPipe()
	if err != nil {
		return 1, err
	}

	err = cmd.Start()
	if err != nil {
		return 1, err
	}

	combined := io.MultiReader(stdout, stderr)
	scanner := bufio.NewScanner(combined)
	for scanner.Scan() {
		fmt.Println(scanner.Text())
	}

	err = cmd.Wait()
	if err != nil {
		return 1, err
	}

	return cmd.ProcessState.ExitCode(), nil
}

func getLatestVersion() (string, error) {
	allVersions, err := getAllVersions()
	if err != nil {
		return "", err
	}

	if len(allVersions) == 0 {
		return "", errors.New("No releases exist")
	}

	return allVersions[0], nil
}

// version is either "latest" or of the form "x.x.x"
func getRelease(version string) (Release, error) {
	versionString := ""
	if version == "latest" {
		latestVersion, err := getLatestVersion()
		if err != nil {
			return Release{}, err
		}
		versionString = "-" + latestVersion
	} else {
		versionString = "-" + version
	}

	allUrls, err := getAllUrls()
	if err != nil {
		return Release{}, err
	}

	mlgUrl := ""
	jarUrl := ""

	osString := "-" + runtime.GOOS
	archString := "-" + runtime.GOARCH

	// The following, for example, looks for a url that contains
	// `mlg`, `-darwin`, `-amd64`, and `-0.11.0` to find the URL.
	//
	// That way, if, in the future, if the order of the name of
	// items in a executable name changes, this code will still
	// be able to identify the URL.
	//
	// That is, `mlg-0.11.0-darwin-amd64` and
	// `mlg-darwin-amd64-0.11.0` will both be identified using the
	// logic below.
	//
	// This provides freedom to change the format of executable
	// names in the future (to a small degree) if needed.

	for _, url := range allUrls {
		if mlgUrl == "" &&
		   strings.Contains(url, "mlg") &&
		   strings.Contains(url, osString) &&
			 strings.Contains(url, archString) &&
			 strings.Contains(url, versionString) {
				 mlgUrl = url
		} else if jarUrl == "" &&
			strings.Contains(url, "mathlingua") &&
			strings.Contains(url, versionString) &&
			strings.HasSuffix(url, ".jar") {
				jarUrl = url
		}
	}

	return Release{
		mlgUrl: mlgUrl,
		jarUrl: jarUrl,
	}, nil
}

func getAllVersions() ([]string, error) {
	allUrls, err := getAllUrls()
	if err != nil {
		return nil, err
	}

	result := []string{}
	for _, url := range allUrls {
		withoutPrefix := strings.ReplaceAll(url,
			"https://github.com/DominicKramer/mathlingua/releases/download/v", "")

		index := strings.Index(withoutPrefix, "/")
		version := ""
		if index < 0 {
			version = withoutPrefix
		} else {
			version = withoutPrefix[:index]
		}

		if len(result) == 0 || result[len(result)-1] != version {
			result = append(result, version)
		}
	}

	return result, nil
}

func getAllUrls() ([]string, error) {
	resp, err := http.Get(
		"https://api.github.com/repos/DominicKramer/mathlingua/releases")
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	var jsonData []map[string]interface{}
	json.Unmarshal(data, &jsonData)

	urls := []string{}
	for _, item := range jsonData {
		castErr := errors.New("Could not determine download url")

		assets, ok := item["assets"].([]interface{})
		if !ok {
			return nil, castErr
		}

		if len(assets) == 0 {
			return nil, castErr
		}

		for _, asset := range assets {
			resultMap, ok := asset.(map[string]interface{})
			if !ok {
				return nil, castErr
			}

			browserDownloadUrl := resultMap["browser_download_url"].(string)
			if !ok {
				return nil, castErr
			}

			urls = append(urls, browserDownloadUrl)
		}
	}

	return urls, nil
}

func downloadUrl(url string, to string) error {
	outPath := to + ".part"
	outputFile, err := os.Create(outPath)
	if err != nil {
		return err
	}
	defer outputFile.Close()

	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	_, err = io.Copy(outputFile, resp.Body)
	if err != nil {
		return err
	}

	return os.Rename(outPath, to)
}

func ensureMathLinguaJarExists(programName string, programArgs []string,
	version string, isUpdating bool) (bool, error) {
	binDir := ".bin"
	_, err := os.Stat(binDir)
	if os.IsNotExist(err) {
		err := os.MkdirAll(binDir, 0755)
		if err != nil {
			return false, err
		}
	}

	jarfile := path.Join(binDir, "mathlingua.jar")
	_, err = os.Stat(jarfile)
	if os.IsNotExist(err) {
		if !isUpdating {
			fmt.Println("Initial run of mlg detected...")
			fmt.Println()
		}

		if version == "latest" {
			fmt.Println("Downloading the latest version of MathLingua core...")
		} else {
			fmt.Println("Downloading version " + version + " of MathLingua core...")
		}

		release, err := getRelease(version)
		if err != nil {
			return false, err
		}

		if release.jarUrl == "" {
			if version == "latest" {
				return false, errors.New("ERROR: Unable to find the latest version of MathLingua")
			} else {
				return false, errors.New("ERROR: Unable to find version " + version + " of MathLingua")
			}
		}

		// download the mathlingua.jar file
		downloadUrl(release.jarUrl, path.Join(".bin", "mathlingua.jar"))

		// if the user is not running `mlg update`, don't attempt to
		// replace the mlg script.  We just want to get a jarfile
		// that aligns with the version of the `mlg` exectuable
		// the user already has
		if !isUpdating {
			return true, nil
		}

		if release.mlgUrl == "" {
			if version == "latest" {
				fmt.Println("WARNING: Unable to find the latest version of mlg")
			} else {
				fmt.Println("WARNING: Unable to find version " + version + " of mlg")
			}
		}

		// if the version of mlg could not be found continue with the
		// existing version of mlg and run the jarfile.
		// Note: a warning is still printed above in this case.
		if release.mlgUrl == "" {
			return true, nil
		}

		// otherwise, the version of mlg could be found, so install it
		newMlgPath, err := getNewMlgPath()
		if err != nil {
			return false, err
		}

		downloadUrl(release.mlgUrl, newMlgPath)
		os.Chmod(newMlgPath, 0755)

		_, err = runCommand(newMlgPath, programArgs)
		return false, err
	}

	return true, nil
}

func copy(src, dest string) error {
	from, err := os.Open(src)
	if err != nil {
		return err
	}
	defer from.Close()

	to, err := os.Create(dest)
	if err != nil {
		return err
	}
	defer to.Close()

	_, err = io.Copy(to, from)
	return err
}

func main() {
	programName := filepath.Base(os.Args[0])

	newMlgPath, err := getNewMlgPath()
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	if programName == "mlg.new" {
		copy(newMlgPath, "mlg")
	} else {
		_, err := os.Stat(newMlgPath)
		if err == nil {
			os.Remove(newMlgPath)
		}
	}

	args := os.Args[1:]
	// when mlg is initially run, it will download version MATHLINGUA_VERSION
	version := MATHLINGUA_VERSION
	isUpdating := false
	if len(args) >= 1 {
		if args[0] == "update" {
			isUpdating = true

			// remove the .bin directory so that the specified
			// version or the latest version is downloaded
			os.RemoveAll(".bin")

			if len(args) >= 2 {
				// use the version given if specified
				if strings.HasPrefix(args[1], "v") {
					version = args[1][1:]
				} else {
					version = args[1]
				}
			} else {
				// otherwise if an explicit version isn't given, update to the latest
				version = "latest"
			}
		} else if args[0] == "versions" {
			versions, err := getAllVersions()
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}

			for index, version := range versions {
				if index == 0 {
					fmt.Println(version + " (latest)")
				} else {
					fmt.Println(version)
				}
			}

			os.Exit(0)
		}
	}

	programArgs := args
	if isUpdating {
		programArgs = []string{"version"}
	}

	runJarFile, err := ensureMathLinguaJarExists(programName, programArgs, version, isUpdating)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	if !runJarFile {
		os.Exit(0)
	}

	exitCode, err := runMathLingua(programArgs)
	if err != nil {
		fmt.Println(err)
	}
	os.Exit(exitCode)
}
