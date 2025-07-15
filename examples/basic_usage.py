#!/usr/bin/env python3
"""
TRN Library Basic Usage Examples

This file demonstrates how to use the TRN (Tool Resource Name) library
for parsing, validating, and manipulating TRN identifiers.

Run this file to see examples of:
- Creating and parsing TRN objects
- Validation and error handling  
- URL conversion
- Pattern matching and utilities
"""

import sys
import os

# Add the parent directory to the path so we can import trn
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import trn
from trn import TRN, TRNBuilder, TRNMatcher
from trn.exceptions import TRNError, TRNValidationError
from trn.utils import (
    normalize_trn, generate_trn, extract_base_trn, 
    is_valid_trn, match_trn_pattern, compare_trn_versions,
    find_matching_trns, group_trns_by_base, get_latest_version_trn
)


def example_basic_parsing():
    """Example: Basic TRN parsing and creation."""
    print("=== Basic TRN Parsing and Creation ===")
    
    # Parse a TRN string
    trn_string = "trn:user:alice:tool:openapi:github-api:v1.0"
    parsed_trn = TRN.parse(trn_string)
    
    print(f"Original: {trn_string}")
    print(f"Platform: {parsed_trn.platform}")
    print(f"Scope: {parsed_trn.scope}")
    print(f"Resource Type: {parsed_trn.resource_type}")
    print(f"Type: {parsed_trn.type}")
    print(f"Instance ID: {parsed_trn.instance_id}")
    print(f"Version: {parsed_trn.version}")
    print(f"Reconstructed: {str(parsed_trn)}")
    
    # Create a TRN object directly
    new_trn = TRN(
        platform="aiplatform",
        resource_type="tool",
        type="workflow",
        instance_id="data-pipeline",
        version="v2.1",
        tag="stable"
    )
    print(f"Created TRN: {new_trn}")
    print()


def example_validation_and_errors():
    """Example: TRN validation and error handling."""
    print("=== TRN Validation and Error Handling ===")
    
    valid_trns = [
        "trn:user:alice:tool:openapi:github-api:v1.0",
        "trn:aiplatform:tool:python:data-processor:latest",
        "trn:org:company:tool:workflow:etl-pipeline:v2.0:production@sha256:abc123"
    ]
    
    invalid_trns = [
        "invalid-trn",
        "trn:invalid-platform:tool:openapi:test:v1.0",
        "trn:user:alice:tool:openapi:github-api",  # Missing version
        "trn:user::tool:openapi:github-api:v1.0",  # Empty scope
    ]
    
    print("Valid TRNs:")
    for trn_str in valid_trns:
        try:
            trn.validate(trn_str)
            print(f"  ✅ {trn_str}")
        except TRNError as e:
            print(f"  ❌ {trn_str} - {e}")
    
    print("\nInvalid TRNs:")
    for trn_str in invalid_trns:
        try:
            trn.validate(trn_str)
            print(f"  ✅ {trn_str}")
        except TRNError as e:
            print(f"  ❌ {trn_str} - {e}")
    
    print()


def example_url_conversion():
    """Example: Converting TRNs to URLs and back."""
    print("=== URL Conversion ===")
    
    # Create a TRN
    test_trn = TRN.parse("trn:user:alice:tool:openapi:async:github-api:v1.0:beta")
    
    # Convert to different URL formats
    trn_url = test_trn.to_url()
    http_url = trn.TRNURLConverter.to_url(test_trn, scheme="https")
    frontend_url = trn.TRNURLConverter.get_frontend_url(test_trn, action="test")
    
    print(f"Original TRN: {test_trn}")
    print(f"TRN URL: {trn_url}")
    print(f"HTTP URL: {http_url}")
    print(f"Frontend URL: {frontend_url}")
    
    # Convert URLs back to TRN
    try:
        trn_from_url = trn.from_url(trn_url)
        print(f"Parsed from URL: {trn_from_url}")
        print(f"Round-trip successful: {str(test_trn) == str(trn_from_url)}")
    except Exception as e:
        print(f"URL parsing failed: {e}")
    
    print()


def example_pattern_matching():
    """Example: TRN pattern matching and filtering."""
    print("=== Pattern Matching and Filtering ===")
    
    trn_list = [
        "trn:user:alice:tool:openapi:github-api:v1.0",
        "trn:user:alice:tool:openapi:slack-api:v2.0",
        "trn:user:bob:tool:python:data-processor:v1.5",
        "trn:org:company:tool:workflow:etl-pipeline:v3.0",
        "trn:aiplatform:tool:shell:file-manager:v1.2"
    ]
    
    print("All TRNs:")
    for t in trn_list:
        print(f"  {t}")
    
    # Pattern matching examples
    patterns = [
        "trn:user:*:tool:openapi:*",  # All user OpenAPI tools
        "trn:user:alice:*",           # All of Alice's tools
        "trn:*:*:tool:*:*:v1.*",     # All v1.x versions
        "trn:org:*:*"                # All organization tools
    ]
    
    for pattern in patterns:
        matches = find_matching_trns(trn_list, pattern)
        print(f"\nPattern '{pattern}' matches {len(matches)} TRNs:")
        for match in matches:
            print(f"  {match}")
    
    print()


def example_version_comparison():
    """Example: Version comparison and management."""
    print("=== Version Comparison and Management ===")
    
    # Version comparison examples
    version_pairs = [
        ("v1.0", "v1.1", ">"),
        ("v2.0", "v1.9", ">"),
        ("v1.2.0", "v1.2.1", "<"),
        ("latest", "stable", ">"),
        ("v1.5", "v1.5", "=="),
        ("v2.0", "v2.*", "~")
    ]
    
    print("Version comparisons:")
    for v1, v2, op in version_pairs:
        result = compare_trn_versions(v1, v2, op)
        print(f"  {v1} {op} {v2}: {result}")
    
    # Group TRNs by base and find latest versions
    versioned_trns = [
        "trn:user:alice:tool:openapi:github-api:v1.0",
        "trn:user:alice:tool:openapi:github-api:v1.1",
        "trn:user:alice:tool:openapi:github-api:v2.0",
        "trn:user:alice:tool:python:data-processor:v0.9",
        "trn:user:alice:tool:python:data-processor:v1.0"
    ]
    
    print(f"\nGrouping {len(versioned_trns)} TRNs by base:")
    groups = group_trns_by_base(versioned_trns)
    
    for base_trn, versions in groups.items():
        latest = get_latest_version_trn(versions)
        print(f"  {base_trn}:")
        print(f"    Versions: {[v.split(':')[-1] for v in versions]}")
        print(f"    Latest: {latest.split(':')[-1] if latest else 'None'}")
    
    print()


def example_builder_pattern():
    """Example: Using the TRN builder pattern."""
    print("=== TRN Builder Pattern ===")
    
    # Build a TRN using the fluent interface
    built_trn = (TRNBuilder()
                  .platform("user")
                  .scope("alice")
                  .resource_type("tool")
                  .type("openapi")
                  .subtype("async")
                  .instance_id("github-api")
                  .version("v1.0")
                  .tag("beta")
                  .hash("sha256:abc123def456")
                  .build())
    
    print(f"Built TRN: {built_trn}")
    
    # Build just the string
    trn_string = (TRNBuilder()
                  .platform("aiplatform")
                  .resource_type("tool")
                  .type("workflow")
                  .instance_id("data-pipeline")
                  .version("latest")
                  .build_string())
    
    print(f"Built string: {trn_string}")
    print()


def example_advanced_matching():
    """Example: Advanced pattern matching with TRNMatcher."""
    print("=== Advanced Pattern Matching ===")
    
    test_trns = [
        "trn:user:alice:tool:openapi:github-api:v1.0",
        "trn:user:alice:tool:openapi:github-api:v1.1",
        "trn:user:alice:tool:openapi:github-api:v2.0",
        "trn:user:bob:tool:python:data-processor:v1.0",
        "trn:org:company:tool:workflow:etl:v1.0"
    ]
    
    # Create matchers for different patterns
    matchers = [
        ("Alice's tools", TRNMatcher("trn:user:alice:*")),
        ("OpenAPI tools", TRNMatcher("trn:*:*:tool:openapi:*")),
        ("Version 1.x", TRNMatcher("trn:*:*:*:*:*:v1.*")),
        ("GitHub API", TRNMatcher("trn:*:*:*:*:github-api:*"))
    ]
    
    for name, matcher in matchers:
        print(f"{name}:")
        matches = [trn for trn in test_trns if matcher.matches(trn)]
        for match in matches:
            print(f"  {match}")
        print()


def example_utility_functions():
    """Example: Various utility functions."""
    print("=== Utility Functions ===")
    
    # Normalization
    messy_trn = "TRN:USER:Alice:TOOL:OpenAPI:GitHub-API:V1.0"
    normalized = normalize_trn(messy_trn)
    print(f"Normalized: {messy_trn} → {normalized}")
    
    # Generation
    generated = generate_trn(
        platform="user",
        resource_type="tool", 
        type="openapi",
        instance_id="new-api",
        version="v1.0",
        scope="alice"
    )
    print(f"Generated: {generated}")
    
    # Base extraction
    full_trn = "trn:user:alice:tool:openapi:github-api:v1.0:beta@sha256:abc123"
    base = extract_base_trn(full_trn)
    print(f"Base TRN: {full_trn} → {base}")
    
    # Validation check
    test_strings = [
        "trn:user:alice:tool:openapi:github-api:v1.0",
        "invalid-trn-string",
        "trn:user:alice:tool:openapi:github-api"  # Missing version
    ]
    
    print("\nValidation checks:")
    for test_str in test_strings:
        valid = is_valid_trn(test_str)
        print(f"  {test_str}: {'✅ Valid' if valid else '❌ Invalid'}")
    
    print()


def example_error_handling():
    """Example: Comprehensive error handling."""
    print("=== Error Handling ===")
    
    error_cases = [
        ("Empty string", ""),
        ("Invalid format", "not-a-trn"),
        ("Reserved word", "trn:user:alice:tool:internal:test:v1.0"),
        ("Too long", "trn:" + "x" * 300),
        ("Invalid platform", "trn:invalid:tool:openapi:test:v1.0"),
        ("Missing version", "trn:user:alice:tool:openapi:github-api"),
    ]
    
    for description, trn_str in error_cases:
        try:
            TRN.parse(trn_str)
            print(f"  {description}: ✅ Parsed successfully")
        except TRNError as e:
            print(f"  {description}: ❌ {type(e).__name__}: {e}")
        except Exception as e:
            print(f"  {description}: ❌ Unexpected error: {e}")
    
    print()


def main():
    """Run all examples."""
    print("TRN Library Usage Examples")
    print("=" * 50)
    print()
    
    examples = [
        example_basic_parsing,
        example_validation_and_errors,
        example_url_conversion,
        example_pattern_matching,
        example_version_comparison,
        example_builder_pattern,
        example_advanced_matching,
        example_utility_functions,
        example_error_handling
    ]
    
    for example_func in examples:
        try:
            example_func()
        except Exception as e:
            print(f"Error in {example_func.__name__}: {e}")
            print()
    
    print("Examples completed!")


if __name__ == "__main__":
    main() 